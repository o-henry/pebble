use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::{
    ai_runtime::AiProvider, monitoring::MonitoringState, region_selection_types::PhysicalRegion,
    watch_intent::CompiledWatchIntent,
};

use super::{
    analysis_interval_ticks, semantic_fingerprint, SmartWatchStatus, SmartWatchTargetStatus,
    WatchAnalysisContext,
};

pub(super) const MAX_WATCH_TARGETS: usize = 3;

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
}

#[derive(Debug, Clone)]
struct WatchEventFingerprint {
    value: String,
    tick: u64,
}

impl WatchTargetRegistry {
    pub(super) fn new(fallback: WatchTargetConfig) -> Self {
        Self {
            current_revision: 0,
            next_id: 1,
            targets: Vec::new(),
            fallback,
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
        });
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
}

fn same_bound_region(left: &PhysicalRegion, right: &PhysicalRegion) -> bool {
    left.monitor_id == right.monitor_id
        && left.x == right.x
        && left.y == right.y
        && left.width == right.width
        && left.height == right.height
        && left.source_window == right.source_window
}
