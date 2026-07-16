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

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledWatchIntent {
    intent: String,
    rule: WatchRule,
}

#[derive(Debug, Clone, PartialEq)]
enum WatchRule {
    Semantic,
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

    pub fn rule_summary(&self) -> String {
        match &self.rule {
            WatchRule::Semantic => "AI SEMANTIC MATCH".to_string(),
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
