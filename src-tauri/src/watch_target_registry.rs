use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::{
    ai_runtime::AiProvider,
    monitoring::MonitoringState,
    region_selection_types::PhysicalRegion,
    visual_loop::{VisualFingerprint, VisualLoopDetector},
    watch_intent::{CompiledWatchIntent, CrossRegionState, FollowThroughRole, LocalWatchMatch},
};

use super::{
    analysis_interval_ticks, semantic_fingerprint, SmartWatchStatus, SmartWatchTargetStatus,
    WatchAnalysisContext,
};

pub(super) const MAX_WATCH_TARGETS: usize = 3;
const CROSS_CONFLICT_CONFIRMATION_TICKS: u64 = 2;

#[derive(Debug, Clone)]
pub(crate) struct WatchRegionAuthorization {
    pub revision: u64,
    pub region: PhysicalRegion,
    pub scale_factor: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct WatchCaptureTarget {
    pub id: String,
    pub name: String,
    pub revision: u64,
    pub region: PhysicalRegion,
    pub scale_factor: f64,
    pub monitoring: MonitoringState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CrossRegionMatch {
    pub regions: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FollowThroughMatch {
    pub regions: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub(crate) struct WatchAuthorization {
    active: Arc<AtomicBool>,
}

impl WatchAuthorization {
    fn new() -> Self {
        Self {
            active: Arc::new(AtomicBool::new(true)),
        }
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    fn revoke(&self) {
        self.active.store(false, Ordering::Release);
    }
}

#[derive(Debug, Clone)]
pub(super) struct WatchTargetConfig {
    pub provider: AiProvider,
    pub model: String,
    pub plan: CompiledWatchIntent,
    pub custom_intent: bool,
    pub locale: String,
    pub analysis_interval_minutes: u16,
    pub ai_fallback_enabled: bool,
}

#[derive(Debug)]
pub(super) struct WatchTargetRegistry {
    current_revision: u64,
    next_id: u64,
    targets: Vec<WatchTarget>,
    fallback: WatchTargetConfig,
    pending_cross_conflict: Option<CrossConflictCandidate>,
    pending_follow_through: Option<FollowThroughCandidate>,
}

#[derive(Debug)]
struct WatchTarget {
    id: String,
    name: String,
    revision: u64,
    region: PhysicalRegion,
    scale_factor: f64,
    authorization: WatchAuthorization,
    monitoring: MonitoringState,
    config: WatchTargetConfig,
    analyses_completed: u32,
    local_matches_completed: u32,
    suppressed_events: u32,
    analysis_in_flight: bool,
    last_analysis_tick: Option<u64>,
    last_event: Option<WatchEventFingerprint>,
    visual_activity_streak: u8,
    visual_activity_observed: bool,
    stability_started_tick: Option<u64>,
    stuck_notified: bool,
    cross_region_state: Option<CrossRegionState>,
    visual_loop: VisualLoopDetector,
}

#[derive(Debug, Clone)]
struct WatchEventFingerprint {
    value: String,
    tick: u64,
}

#[derive(Debug)]
struct CrossConflictCandidate {
    fingerprint: String,
    first_tick: u64,
    notified: bool,
}

struct CrossConflictSnapshot {
    fingerprint: String,
    regions: Vec<String>,
    locale: String,
}

#[derive(Debug)]
struct FollowThroughCandidate {
    trigger_id: String,
    trigger_name: String,
    waiting_results: Vec<FollowThroughResult>,
    deadline_tick: u64,
    deadline_minutes: u16,
    locale: String,
}

#[derive(Debug)]
struct FollowThroughResult {
    id: String,
    name: String,
}

impl WatchTargetRegistry {
    pub(super) fn new(fallback: WatchTargetConfig) -> Self {
        Self {
            current_revision: 0,
            next_id: 1,
            targets: Vec::new(),
            fallback,
            pending_cross_conflict: None,
            pending_follow_through: None,
        }
    }

    pub(super) fn select_current(&mut self, revision: u64) {
        self.current_revision = revision;
    }

    pub(super) fn upsert(
        &mut self,
        authorization: WatchRegionAuthorization,
        config: WatchTargetConfig,
    ) -> Result<(), ()> {
        self.current_revision = authorization.revision;
        self.fallback = config.clone();
        if let Some(target) = self.targets.iter_mut().find(|target| {
            target.revision == authorization.revision
                || same_bound_region(&target.region, &authorization.region)
        }) {
            target.authorization.revoke();
            target.revision = authorization.revision;
            target.region = authorization.region;
            target.scale_factor = authorization.scale_factor;
            target.authorization = WatchAuthorization::new();
            target.monitoring.reset();
            target.config = config;
            target.analysis_in_flight = false;
            target.last_analysis_tick = None;
            target.last_event = None;
            target.reset_visual_activity();
            target.cross_region_state = None;
            target.visual_loop.reset();
            self.pending_cross_conflict = None;
            self.pending_follow_through = None;
            return Ok(());
        }
        if self.targets.len() >= MAX_WATCH_TARGETS {
            return Err(());
        }
        let id = format!("watch-{}", self.next_id);
        let name = format!("REGION {}", self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.targets.push(WatchTarget {
            id,
            name,
            revision: authorization.revision,
            region: authorization.region,
            scale_factor: authorization.scale_factor,
            authorization: WatchAuthorization::new(),
            monitoring: MonitoringState::default(),
            config,
            analyses_completed: 0,
            local_matches_completed: 0,
            suppressed_events: 0,
            analysis_in_flight: false,
            last_analysis_tick: None,
            last_event: None,
            visual_activity_streak: 0,
            visual_activity_observed: false,
            stability_started_tick: None,
            stuck_notified: false,
            cross_region_state: None,
            visual_loop: VisualLoopDetector::default(),
        });
        self.pending_cross_conflict = None;
        self.pending_follow_through = None;
        Ok(())
    }

    pub(super) fn remove_current(&mut self) {
        let revision = self.current_revision;
        self.remove_where(|target| target.revision == revision);
    }

    pub(super) fn remove_target(&mut self, id: &str) -> bool {
        let before = self.targets.len();
        self.remove_where(|target| target.id == id);
        self.targets.len() != before
    }

    pub(super) fn remove_all(&mut self) {
        for target in &self.targets {
            target.authorization.revoke();
        }
        self.targets.clear();
        self.pending_cross_conflict = None;
        self.pending_follow_through = None;
    }

    pub(super) fn capture_targets(&self) -> Vec<WatchCaptureTarget> {
        self.targets
            .iter()
            .map(|target| WatchCaptureTarget {
                id: target.id.clone(),
                name: target.name.clone(),
                revision: target.revision,
                region: target.region.clone(),
                scale_factor: target.scale_factor,
                monitoring: target.monitoring.clone(),
            })
            .collect()
    }

    pub(super) fn contains(&self, id: &str) -> bool {
        self.targets.iter().any(|target| target.id == id)
    }

    pub(super) fn current_context(&self, id: &str) -> Option<WatchAnalysisContext> {
        let target = self.targets.iter().find(|target| target.id == id)?;
        Some(target.context())
    }

    pub(super) fn begin_analysis(&mut self, id: &str, tick: u64) -> Option<WatchAnalysisContext> {
        let target = self.targets.iter_mut().find(|target| target.id == id)?;
        if target.analysis_in_flight || !target.analysis_interval_elapsed(tick) {
            return None;
        }
        target.analysis_in_flight = true;
        target.last_analysis_tick = Some(tick);
        Some(target.context())
    }

    pub(super) fn finish_local_match(&mut self, id: &str, fingerprint: &str, tick: u64) -> bool {
        let Some(target) = self.targets.iter_mut().find(|target| target.id == id) else {
            return false;
        };
        if !target.accept_event(fingerprint, tick) {
            return false;
        }
        target.local_matches_completed = target.local_matches_completed.saturating_add(1);
        true
    }

    pub(super) fn reset_visual_activity(&mut self, id: &str) {
        if let Some(target) = self.targets.iter_mut().find(|target| target.id == id) {
            target.reset_visual_activity();
        }
    }

    pub(super) fn note_visual_activity(&mut self, id: &str, tick: u64, settled: bool) {
        if let Some(target) = self.targets.iter_mut().find(|target| target.id == id) {
            target.note_visual_activity(tick, settled);
        }
    }

    pub(super) fn observe_visual_stability(
        &mut self,
        id: &str,
        tick: u64,
    ) -> Option<LocalWatchMatch> {
        self.targets
            .iter_mut()
            .find(|target| target.id == id)?
            .observe_visual_stability(tick)
    }

    pub(super) fn update_cross_region_state(
        &mut self,
        id: &str,
        state: Option<CrossRegionState>,
    ) -> bool {
        let Some(target) = self.targets.iter_mut().find(|target| target.id == id) else {
            return false;
        };
        if !target.config.plan.detects_cross_region_conflict() {
            return false;
        }
        if target.cross_region_state != state {
            target.cross_region_state = state;
            self.pending_cross_conflict = None;
        }
        true
    }

    pub(super) fn observe_cross_region_conflict(&mut self, tick: u64) -> Option<CrossRegionMatch> {
        let Some(conflict) = self.current_cross_conflict() else {
            self.pending_cross_conflict = None;
            return None;
        };
        let should_emit = match self.pending_cross_conflict.as_mut() {
            Some(candidate) if candidate.fingerprint == conflict.fingerprint => {
                if candidate.notified
                    || tick.saturating_sub(candidate.first_tick) < CROSS_CONFLICT_CONFIRMATION_TICKS
                {
                    false
                } else {
                    candidate.notified = true;
                    true
                }
            }
            _ => {
                self.pending_cross_conflict = Some(CrossConflictCandidate {
                    fingerprint: conflict.fingerprint,
                    first_tick: tick,
                    notified: false,
                });
                false
            }
        };
        if !should_emit {
            return None;
        }
        for target in self
            .targets
            .iter_mut()
            .filter(|target| conflict.regions.contains(&target.name))
        {
            target.local_matches_completed = target.local_matches_completed.saturating_add(1);
        }
        Some(CrossRegionMatch {
            summary: cross_conflict_summary(&conflict.locale, &conflict.regions),
            regions: conflict.regions,
        })
    }

    pub(super) fn note_follow_through_change(&mut self, id: &str, tick: u64) -> bool {
        let Some(target) = self.targets.iter().find(|target| target.id == id) else {
            return false;
        };
        let Some(role) = target.config.plan.follow_through_role() else {
            return false;
        };
        match role {
            FollowThroughRole::Trigger => {
                let waiting_results = self
                    .targets
                    .iter()
                    .filter(|target| {
                        target.config.plan.follow_through_role() == Some(FollowThroughRole::Result)
                    })
                    .map(|target| FollowThroughResult {
                        id: target.id.clone(),
                        name: target.name.clone(),
                    })
                    .collect::<Vec<_>>();
                if waiting_results.is_empty() {
                    self.pending_follow_through = None;
                    return true;
                }
                self.pending_follow_through = Some(FollowThroughCandidate {
                    trigger_id: target.id.clone(),
                    trigger_name: target.name.clone(),
                    waiting_results,
                    deadline_tick: tick.saturating_add(analysis_interval_ticks(
                        target.config.analysis_interval_minutes,
                    )),
                    deadline_minutes: target.config.analysis_interval_minutes,
                    locale: target.config.locale.clone(),
                });
            }
            FollowThroughRole::Result => {
                let Some(candidate) = self.pending_follow_through.as_mut() else {
                    return true;
                };
                candidate.waiting_results.retain(|result| result.id != id);
                if candidate.waiting_results.is_empty() {
                    self.pending_follow_through = None;
                }
            }
        }
        true
    }

    pub(super) fn observe_follow_through_deadline(
        &mut self,
        tick: u64,
    ) -> Option<FollowThroughMatch> {
        let candidate = self.pending_follow_through.as_ref()?;
        if tick < candidate.deadline_tick {
            return None;
        }
        let candidate = self.pending_follow_through.take()?;
        let mut regions = vec![candidate.trigger_name.clone()];
        regions.extend(
            candidate
                .waiting_results
                .iter()
                .map(|result| result.name.clone()),
        );
        for target in self.targets.iter_mut().filter(|target| {
            target.id == candidate.trigger_id
                || candidate
                    .waiting_results
                    .iter()
                    .any(|result| result.id == target.id)
        }) {
            target.local_matches_completed = target.local_matches_completed.saturating_add(1);
        }
        Some(FollowThroughMatch {
            summary: follow_through_summary(
                &candidate.locale,
                &candidate.trigger_name,
                &candidate
                    .waiting_results
                    .iter()
                    .map(|result| result.name.clone())
                    .collect::<Vec<_>>(),
                candidate.deadline_minutes,
            ),
            regions,
        })
    }

    pub(super) fn invalidate_local_relationships(&mut self, id: &str) {
        if let Some(target) = self.targets.iter_mut().find(|target| target.id == id) {
            if target.config.plan.detects_cross_region_conflict() {
                target.cross_region_state = None;
                self.pending_cross_conflict = None;
            }
            target.visual_loop.reset();
        }
        if self
            .pending_follow_through
            .as_ref()
            .is_some_and(|candidate| {
                candidate.trigger_id == id
                    || candidate
                        .waiting_results
                        .iter()
                        .any(|result| result.id == id)
            })
        {
            self.pending_follow_through = None;
        }
    }

    pub(super) fn seed_visual_loop(&mut self, id: &str, fingerprint: VisualFingerprint) -> bool {
        let Some(target) = self.targets.iter_mut().find(|target| target.id == id) else {
            return false;
        };
        if !target.config.plan.detects_visual_loop() {
            return false;
        }
        target.visual_loop.seed(fingerprint);
        true
    }

    pub(super) fn observe_visual_loop(
        &mut self,
        id: &str,
        fingerprint: VisualFingerprint,
    ) -> Option<LocalWatchMatch> {
        let target = self.targets.iter_mut().find(|target| target.id == id)?;
        if !target.config.plan.detects_visual_loop() {
            return None;
        }
        let pattern = target.visual_loop.observe(fingerprint)?;
        target.local_matches_completed = target.local_matches_completed.saturating_add(1);
        Some(LocalWatchMatch {
            summary: visual_loop_summary(&target.config.locale, pattern.period),
            fingerprint: format!("local:visual-loop:{}", pattern.period),
        })
    }

    pub(super) fn finish_analysis(&mut self, id: &str, completed: bool) -> bool {
        let Some(target) = self.targets.iter_mut().find(|target| target.id == id) else {
            return false;
        };
        target.analysis_in_flight = false;
        if completed {
            target.analyses_completed = target.analyses_completed.saturating_add(1);
        }
        true
    }

    pub(super) fn accept_ai_event(&mut self, id: &str, summary: &str, tick: u64) -> bool {
        let Some(target) = self.targets.iter_mut().find(|target| target.id == id) else {
            return false;
        };
        target.accept_event(&semantic_fingerprint(summary), tick)
    }

    pub(super) fn set_current_interval(&mut self, minutes: u16) -> bool {
        self.fallback.analysis_interval_minutes = minutes;
        let Some(target) = self
            .targets
            .iter_mut()
            .find(|target| target.revision == self.current_revision)
        else {
            return false;
        };
        target.config.analysis_interval_minutes = minutes;
        true
    }

    pub(super) fn status(&self) -> SmartWatchStatus {
        let current = self
            .targets
            .iter()
            .find(|target| target.revision == self.current_revision);
        let config = current
            .map(|target| &target.config)
            .unwrap_or(&self.fallback);
        SmartWatchStatus {
            enabled: current.is_some(),
            target_count: self.targets.len() as u8,
            targets: self
                .targets
                .iter()
                .map(|target| target.status(self.current_revision))
                .collect(),
            analyses_completed: current.map_or(0, |target| target.analyses_completed),
            local_matches_completed: current.map_or(0, |target| target.local_matches_completed),
            suppressed_events: current.map_or(0, |target| target.suppressed_events),
            analysis_interval_minutes: config.analysis_interval_minutes,
            provider: config.provider,
            model: config.model.clone(),
            ai_fallback_enabled: config.ai_fallback_enabled,
            custom_intent: current.is_some_and(|target| target.config.custom_intent),
            watching_for: current.map(|target| target.config.plan.intent().to_string()),
            evaluation_mode: config.plan.mode(),
            local_engine: config.plan.local_engine(),
            rule_summary: config.plan.rule_summary(),
            capture_scope: "selectedRegionOnly",
            storage_policy: "memoryOnly",
            images_saved: false,
            ocr_saved: false,
        }
    }

    fn remove_where(&mut self, predicate: impl Fn(&WatchTarget) -> bool) {
        for target in self.targets.iter().filter(|target| predicate(target)) {
            target.authorization.revoke();
        }
        self.targets.retain(|target| !predicate(target));
        self.pending_cross_conflict = None;
        self.pending_follow_through = None;
    }

    fn current_cross_conflict(&self) -> Option<CrossConflictSnapshot> {
        let mut states = self
            .targets
            .iter()
            .filter(|target| target.config.plan.detects_cross_region_conflict())
            .filter_map(|target| target.cross_region_state.map(|state| (target, state)))
            .collect::<Vec<_>>();
        let has_positive = states
            .iter()
            .any(|(_, state)| *state == CrossRegionState::Positive);
        let has_negative = states
            .iter()
            .any(|(_, state)| *state == CrossRegionState::Negative);
        if !has_positive || !has_negative {
            return None;
        }
        states.sort_by(|(left, _), (right, _)| left.name.cmp(&right.name));
        let fingerprint = states
            .iter()
            .map(|(target, state)| {
                let state = match state {
                    CrossRegionState::Positive => "positive",
                    CrossRegionState::Negative => "negative",
                };
                format!("{}:{state}", target.id)
            })
            .collect::<Vec<_>>()
            .join("|");
        let regions = states
            .iter()
            .map(|(target, _)| target.name.clone())
            .collect();
        let locale = states[0].0.config.locale.clone();
        Some(CrossConflictSnapshot {
            fingerprint,
            regions,
            locale,
        })
    }
}

impl WatchTarget {
    fn context(&self) -> WatchAnalysisContext {
        WatchAnalysisContext {
            provider: self.config.provider,
            model: self.config.model.clone(),
            target_name: self.name.clone(),
            intent: self.config.plan.intent().to_string(),
            locale: self.config.locale.clone(),
            plan: self.config.plan.clone(),
            authorization: self.authorization.clone(),
            ai_fallback_enabled: self.config.ai_fallback_enabled,
        }
    }

    fn status(&self, current_revision: u64) -> SmartWatchTargetStatus {
        SmartWatchTargetStatus {
            id: self.id.clone(),
            name: self.name.clone(),
            current: self.revision == current_revision,
            analyses_completed: self.analyses_completed,
            local_matches_completed: self.local_matches_completed,
            suppressed_events: self.suppressed_events,
            analysis_interval_minutes: self.config.analysis_interval_minutes,
            provider: self.config.provider,
            model: self.config.model.clone(),
            ai_fallback_enabled: self.config.ai_fallback_enabled,
            evaluation_mode: self.config.plan.mode(),
            local_engine: self.config.plan.local_engine(),
            rule_summary: self.config.plan.rule_summary(),
        }
    }

    fn analysis_interval_elapsed(&self, tick: u64) -> bool {
        self.last_analysis_tick
            .map(|last_tick| {
                tick.saturating_sub(last_tick)
                    >= analysis_interval_ticks(self.config.analysis_interval_minutes)
            })
            .unwrap_or(true)
    }

    fn accept_event(&mut self, fingerprint: &str, tick: u64) -> bool {
        let duplicate = self.last_event.as_ref().is_some_and(|last| {
            last.value == fingerprint
                && tick.saturating_sub(last.tick)
                    < analysis_interval_ticks(self.config.analysis_interval_minutes)
        });
        if duplicate {
            self.suppressed_events = self.suppressed_events.saturating_add(1);
            return false;
        }
        self.last_event = Some(WatchEventFingerprint {
            value: fingerprint.to_string(),
            tick,
        });
        true
    }

    fn reset_visual_activity(&mut self) {
        self.visual_activity_streak = 0;
        self.visual_activity_observed = false;
        self.stability_started_tick = None;
        self.stuck_notified = false;
    }

    fn note_visual_activity(&mut self, tick: u64, settled: bool) {
        if !self.config.plan.detects_stuck_after_activity() {
            return;
        }
        if settled {
            self.visual_activity_streak = 0;
            self.visual_activity_observed = true;
            self.stability_started_tick = Some(tick);
            self.stuck_notified = false;
            return;
        }
        self.visual_activity_streak = self.visual_activity_streak.saturating_add(1);
        self.stability_started_tick = None;
        if self.visual_activity_streak >= 2 {
            self.visual_activity_observed = true;
            self.stuck_notified = false;
        }
    }

    fn observe_visual_stability(&mut self, tick: u64) -> Option<LocalWatchMatch> {
        if !self.config.plan.detects_stuck_after_activity() {
            return None;
        }
        self.visual_activity_streak = 0;
        if !self.visual_activity_observed || self.stuck_notified {
            return None;
        }
        let stable_since = match self.stability_started_tick {
            Some(stable_since) => stable_since,
            None => {
                self.stability_started_tick = Some(tick);
                return None;
            }
        };
        if tick.saturating_sub(stable_since)
            < analysis_interval_ticks(self.config.analysis_interval_minutes)
        {
            return None;
        }
        self.stuck_notified = true;
        let fingerprint = "local:stuck-after-activity";
        if !self.accept_event(fingerprint, tick) {
            return None;
        }
        self.local_matches_completed = self.local_matches_completed.saturating_add(1);
        Some(LocalWatchMatch {
            summary: stuck_summary(&self.config.locale, self.config.analysis_interval_minutes),
            fingerprint: fingerprint.to_string(),
        })
    }
}

fn stuck_summary(locale: &str, minutes: u16) -> String {
    if locale.to_ascii_lowercase().starts_with("ko") {
        let duration = if minutes == 60 {
            "1시간".to_string()
        } else {
            format!("{minutes}분")
        };
        format!("진행되던 화면이 {duration} 동안 바뀌지 않았습니다.")
    } else {
        let duration = if minutes == 60 {
            "1 hour".to_string()
        } else if minutes == 1 {
            "1 minute".to_string()
        } else {
            format!("{minutes} minutes")
        };
        format!(
            "The region stopped changing after visible activity and stayed unchanged for {duration}."
        )
    }
}

fn cross_conflict_summary(locale: &str, regions: &[String]) -> String {
    let regions = regions.join(", ");
    if locale.to_ascii_lowercase().starts_with("ko") {
        format!("{regions}에 성공·정상 상태와 오류·실패 상태가 동시에 유지되고 있습니다.")
    } else {
        format!(
            "Opposing success or healthy and error or failed states remain visible across {regions}."
        )
    }
}

fn follow_through_summary(locale: &str, trigger: &str, results: &[String], minutes: u16) -> String {
    let results = results.join(", ");
    if locale.to_ascii_lowercase().starts_with("ko") {
        let duration = if minutes == 60 {
            "1시간".to_string()
        } else {
            format!("{minutes}분")
        };
        format!("{trigger} 변화 후 {duration} 안에 {results}의 후속 변화가 감지되지 않았습니다.")
    } else {
        let duration = if minutes == 60 {
            "1 hour".to_string()
        } else if minutes == 1 {
            "1 minute".to_string()
        } else {
            format!("{minutes} minutes")
        };
        format!("{results} did not change within {duration} after {trigger} changed.")
    }
}

fn visual_loop_summary(locale: &str, period: usize) -> String {
    if locale.to_ascii_lowercase().starts_with("ko") {
        format!("화면이 {period}단계 패턴을 세 번 연속 반복하고 있습니다.")
    } else {
        format!("The region repeated the same {period}-step visual cycle three times.")
    }
}

fn same_bound_region(left: &PhysicalRegion, right: &PhysicalRegion) -> bool {
    left.monitor_id == right.monitor_id
        && left.x == right.x
        && left.y == right.y
        && left.width == right.width
        && left.height == right.height
        && left.source_window == right.source_window
}
