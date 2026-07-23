use super::parser::extract_numbers;
use super::{
    contains_any, display_number, normalize, LocalWatchDecision, LocalWatchMatch, NumberOperator,
    WatchRule, WatchState,
};

pub(super) fn evaluate(
    rule: &WatchRule,
    previous_text: Option<&str>,
    current_text: Option<&str>,
    locale: &str,
) -> LocalWatchDecision {
    match rule {
        WatchRule::Automatic => {
            return evaluate_automatic(previous_text, current_text, locale);
        }
        WatchRule::StuckAfterActivity
        | WatchRule::CrossRegionConflict
        | WatchRule::FollowThroughTrigger
        | WatchRule::FollowThroughResult
        | WatchRule::VisualLoop => return LocalWatchDecision::NotMatched,
        WatchRule::Semantic => return LocalWatchDecision::NeedsAi,
        _ => {}
    }
    let (Some(previous), Some(current)) = (previous_text, current_text) else {
        return LocalWatchDecision::NeedsAi;
    };
    let previous = normalize(previous);
    let current = normalize(current);

    let matched = match rule {
        WatchRule::Automatic
        | WatchRule::Semantic
        | WatchRule::StuckAfterActivity
        | WatchRule::CrossRegionConflict
        | WatchRule::FollowThroughTrigger
        | WatchRule::FollowThroughResult
        | WatchRule::VisualLoop => unreachable!("handled before OCR evidence validation"),
        WatchRule::TextAppears(text) => !previous.contains(text) && current.contains(text),
        WatchRule::TextDisappears(text) => previous.contains(text) && !current.contains(text),
        WatchRule::TextChanges => !previous.is_empty() && previous != current,
        WatchRule::NumberCrosses(operator, threshold) => {
            return number_decision(rule, *operator, *threshold, &previous, &current, locale);
        }
        WatchRule::ProgressReaches(threshold) => {
            return number_decision(
                rule,
                NumberOperator::AtLeast,
                *threshold,
                &previous,
                &current,
                locale,
            );
        }
        WatchRule::StateAppears(state) => {
            !contains_state(&previous, *state) && contains_state(&current, *state)
        }
    };

    decision_for_match(rule, matched, locale)
}

fn number_decision(
    rule: &WatchRule,
    operator: NumberOperator,
    threshold: f64,
    previous: &str,
    current: &str,
    locale: &str,
) -> LocalWatchDecision {
    let previous_numbers = extract_numbers(previous);
    let current_numbers = extract_numbers(current);
    if previous_numbers.len() != 1 || current_numbers.len() != 1 {
        return LocalWatchDecision::NeedsAi;
    }
    let previous = previous_numbers[0];
    let current = current_numbers[0];
    let matched = match operator {
        NumberOperator::GreaterThan => previous <= threshold && current > threshold,
        NumberOperator::AtLeast => previous < threshold && current >= threshold,
        NumberOperator::LessThan => previous >= threshold && current < threshold,
        NumberOperator::AtMost => previous > threshold && current <= threshold,
    };
    decision_for_match(rule, matched, locale)
}

fn decision_for_match(rule: &WatchRule, matched: bool, locale: &str) -> LocalWatchDecision {
    if !matched {
        return LocalWatchDecision::NotMatched;
    }
    LocalWatchDecision::Matched(LocalWatchMatch {
        summary: local_summary(rule, locale),
        fingerprint: format!("local:{}", rule_summary(rule).to_ascii_lowercase()),
    })
}

fn evaluate_automatic(
    previous_text: Option<&str>,
    current_text: Option<&str>,
    locale: &str,
) -> LocalWatchDecision {
    for rule in [
        WatchRule::TextAppears("build failed".to_string()),
        WatchRule::TextAppears("queue empty".to_string()),
        WatchRule::ProgressReaches(100.0),
        WatchRule::StateAppears(WatchState::Error),
    ] {
        if let LocalWatchDecision::Matched(event) =
            evaluate(&rule, previous_text, current_text, locale)
        {
            return LocalWatchDecision::Matched(event);
        }
    }
    LocalWatchDecision::NeedsAi
}

fn rule_summary(rule: &WatchRule) -> String {
    match rule {
        WatchRule::Automatic => "AUTOMATIC WATCH".to_string(),
        WatchRule::Semantic => "AI SEMANTIC MATCH".to_string(),
        WatchRule::StuckAfterActivity => "NO PROGRESS AFTER ACTIVITY".to_string(),
        WatchRule::CrossRegionConflict => "CROSS-REGION STATUS CONFLICT".to_string(),
        WatchRule::FollowThroughTrigger => "FOLLOW THROUGH TRIGGER".to_string(),
        WatchRule::FollowThroughResult => "FOLLOW THROUGH RESULT".to_string(),
        WatchRule::VisualLoop => "REPEATING VISUAL LOOP".to_string(),
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

fn contains_state(value: &str, state: WatchState) -> bool {
    match state {
        WatchState::Error => contains_any(
            value,
            &["error", "failed", "failure", "오류", "에러", "실패"],
        ),
        WatchState::Warning => contains_any(value, &["warning", "warn", "경고", "주의"]),
        WatchState::Success => contains_any(
            value,
            &["success", "passed", "complete", "completed", "성공", "완료"],
        ),
    }
}

fn local_summary(rule: &WatchRule, locale: &str) -> String {
    let condition = match rule {
        WatchRule::Automatic => "AUTOMATIC WATCH MATCHED".to_string(),
        WatchRule::TextAppears(text) => format!("TEXT APPEARED: {text}"),
        WatchRule::TextDisappears(text) => format!("TEXT DISAPPEARED: {text}"),
        WatchRule::TextChanges => "VISIBLE TEXT CHANGED".to_string(),
        WatchRule::NumberCrosses(operator, threshold) => {
            format!(
                "NUMBER CROSSED {} {}",
                operator.label(),
                display_number(*threshold)
            )
        }
        WatchRule::ProgressReaches(threshold) => {
            format!("PROGRESS REACHED {}%", display_number(*threshold))
        }
        WatchRule::StateAppears(state) => format!("{} STATE APPEARED", state.label()),
        WatchRule::Semantic => "MEANINGFUL CHANGE MATCHED".to_string(),
        WatchRule::StuckAfterActivity => "NO PROGRESS AFTER ACTIVITY".to_string(),
        WatchRule::CrossRegionConflict => "CROSS-REGION STATUS CONFLICT".to_string(),
        WatchRule::FollowThroughTrigger => "FOLLOW THROUGH TRIGGER".to_string(),
        WatchRule::FollowThroughResult => "FOLLOW THROUGH RESULT".to_string(),
        WatchRule::VisualLoop => "REPEATING VISUAL LOOP".to_string(),
    };
    if locale.to_ascii_lowercase().starts_with("ko") {
        format!("조건이 충족되었습니다. {condition}")
    } else {
        format!("Watch condition matched. {condition}")
    }
}
