use serde::Serialize;

#[path = "watch_intent_evaluator.rs"]
mod evaluator;
#[path = "watch_intent_parser.rs"]
mod parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WatchEvaluationMode {
    Local,
    Ai,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WatchLocalEngine {
    Ocr,
    VisualStability,
    CrossRegionOcr,
    FollowThroughTrigger,
    FollowThroughResult,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledWatchIntent {
    intent: String,
    rule: WatchRule,
}

#[derive(Debug, Clone, PartialEq)]
enum WatchRule {
    Semantic,
    StuckAfterActivity,
    CrossRegionConflict,
    FollowThroughTrigger,
    FollowThroughResult,
    TextAppears(String),
    TextDisappears(String),
    TextChanges,
    NumberCrosses(NumberOperator, f64),
    ProgressReaches(f64),
    StateAppears(WatchState),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum NumberOperator {
    GreaterThan,
    AtLeast,
    LessThan,
    AtMost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WatchState {
    Error,
    Warning,
    Success,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalWatchMatch {
    pub summary: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalWatchDecision {
    Matched(LocalWatchMatch),
    NotMatched,
    NeedsAi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossRegionState {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FollowThroughRole {
    Trigger,
    Result,
}

impl CompiledWatchIntent {
    pub fn compile(intent: String) -> Self {
        let normalized = normalize(&intent);
        let rule = parser::compile_rule(&intent, &normalized);
        Self { intent, rule }
    }

    pub fn intent(&self) -> &str {
        &self.intent
    }

    pub fn mode(&self) -> WatchEvaluationMode {
        match self.rule {
            WatchRule::Semantic => WatchEvaluationMode::Ai,
            _ => WatchEvaluationMode::Local,
        }
    }

    pub fn local_engine(&self) -> Option<WatchLocalEngine> {
        match self.rule {
            WatchRule::Semantic => None,
            WatchRule::StuckAfterActivity => Some(WatchLocalEngine::VisualStability),
            WatchRule::CrossRegionConflict => Some(WatchLocalEngine::CrossRegionOcr),
            WatchRule::FollowThroughTrigger => Some(WatchLocalEngine::FollowThroughTrigger),
            WatchRule::FollowThroughResult => Some(WatchLocalEngine::FollowThroughResult),
            _ => Some(WatchLocalEngine::Ocr),
        }
    }

    pub fn detects_stuck_after_activity(&self) -> bool {
        matches!(self.rule, WatchRule::StuckAfterActivity)
    }

    pub fn detects_cross_region_conflict(&self) -> bool {
        matches!(self.rule, WatchRule::CrossRegionConflict)
    }

    pub fn follow_through_role(&self) -> Option<FollowThroughRole> {
        match self.rule {
            WatchRule::FollowThroughTrigger => Some(FollowThroughRole::Trigger),
            WatchRule::FollowThroughResult => Some(FollowThroughRole::Result),
            _ => None,
        }
    }

    pub fn classify_cross_region_state(&self, text: &str) -> Option<CrossRegionState> {
        if !self.detects_cross_region_conflict() {
            return None;
        }
        let normalized = normalize(text);
        if contains_negated_ascii_status(
            &normalized,
            &[
                "success",
                "succeeded",
                "successful",
                "passed",
                "complete",
                "completed",
                "ready",
                "healthy",
                "online",
                "operational",
                "up",
            ],
        ) || contains_any(
            &normalized,
            &["성공하지 않", "완료되지 않", "정상 아님", "정상이 아님"],
        ) || contains_status_any(
            &normalized,
            &[
                "error",
                "errors",
                "failed",
                "failure",
                "fail",
                "offline",
                "unhealthy",
                "blocked",
                "rejected",
                "down",
                "unsuccessful",
                "오류",
                "에러",
                "실패",
                "비정상",
                "오프라인",
                "중단",
                "거부",
                "차단",
                "미완료",
            ],
        ) {
            return Some(CrossRegionState::Negative);
        }
        contains_status_any(
            &normalized,
            &[
                "success",
                "succeeded",
                "successful",
                "passed",
                "complete",
                "completed",
                "ready",
                "healthy",
                "online",
                "operational",
                "up",
                "성공",
                "완료",
                "정상",
                "온라인",
                "통과",
                "준비",
            ],
        )
        .then_some(CrossRegionState::Positive)
    }

    pub fn rule_summary(&self) -> String {
        match &self.rule {
            WatchRule::Semantic => "AI SEMANTIC MATCH".to_string(),
            WatchRule::StuckAfterActivity => "NO PROGRESS AFTER ACTIVITY".to_string(),
            WatchRule::CrossRegionConflict => "CROSS-REGION STATUS CONFLICT".to_string(),
            WatchRule::FollowThroughTrigger => "FOLLOW THROUGH TRIGGER".to_string(),
            WatchRule::FollowThroughResult => "FOLLOW THROUGH RESULT".to_string(),
            WatchRule::TextAppears(text) => format!("TEXT APPEARS: {text}"),
            WatchRule::TextDisappears(text) => format!("TEXT DISAPPEARS: {text}"),
            WatchRule::TextChanges => "VISIBLE TEXT CHANGES".to_string(),
            WatchRule::NumberCrosses(operator, threshold) => {
                format!("NUMBER {} {}", operator.label(), display_number(*threshold))
            }
            WatchRule::ProgressReaches(threshold) => {
                format!("PROGRESS REACHES {}%", display_number(*threshold))
            }
            WatchRule::StateAppears(state) => format!("{} STATE APPEARS", state.label()),
        }
    }

    pub fn evaluate(
        &self,
        previous_text: Option<&str>,
        current_text: Option<&str>,
        locale: &str,
    ) -> LocalWatchDecision {
        evaluator::evaluate(&self.rule, previous_text, current_text, locale)
    }
}

impl NumberOperator {
    fn label(self) -> &'static str {
        match self {
            Self::GreaterThan => ">",
            Self::AtLeast => ">=",
            Self::LessThan => "<",
            Self::AtMost => "<=",
        }
    }
}

impl WatchState {
    fn label(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warning => "WARNING",
            Self::Success => "SUCCESS",
        }
    }
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn contains_status_any(value: &str, terms: &[&str]) -> bool {
    let words = value
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();
    terms.iter().any(|term| {
        if term.is_ascii() {
            words.contains(term)
        } else {
            value.contains(term)
        }
    })
}

fn contains_negated_ascii_status(value: &str, terms: &[&str]) -> bool {
    let words = value
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();
    words
        .windows(2)
        .any(|pair| matches!(pair[0], "not" | "no") && terms.contains(&pair[1]))
}

fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn display_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}
