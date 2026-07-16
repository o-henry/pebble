use super::{contains_any, normalize, NumberOperator, WatchRule, WatchState};

pub(super) fn compile_rule(original: &str, normalized: &str) -> WatchRule {
    if is_visual_loop_intent(normalized) {
        return WatchRule::VisualLoop;
    }
    if is_follow_through_trigger(normalized) {
        return WatchRule::FollowThroughTrigger;
    }
    if is_follow_through_result(normalized) {
        return WatchRule::FollowThroughResult;
    }
    if is_cross_region_conflict_intent(normalized) {
        return WatchRule::CrossRegionConflict;
    }
    if is_stuck_intent(normalized) {
        return WatchRule::StuckAfterActivity;
    }
    if is_text_change_intent(normalized) {
        return WatchRule::TextChanges;
    }
    if let Some((operator, threshold)) = numeric_rule(normalized) {
        return if is_progress_intent(normalized) {
            WatchRule::ProgressReaches(threshold)
        } else {
            WatchRule::NumberCrosses(operator, threshold)
        };
    }
    if let Some(text) = text_rule(original, normalized, false) {
        return WatchRule::TextDisappears(text);
    }
    if let Some(text) = text_rule(original, normalized, true) {
        return WatchRule::TextAppears(text);
    }
    if let Some(state) = state_rule(normalized) {
        return WatchRule::StateAppears(state);
    }
    WatchRule::Semantic
}

fn is_visual_loop_intent(value: &str) -> bool {
    contains_any(
        value,
        &[
            "repeats the same visual cycle",
            "repeating visual loop",
            "visual loop detector",
            "screen keeps looping",
            "same screen cycle repeats",
            "화면 반복 루프",
            "같은 화면 순환 반복",
            "화면이 계속 반복",
        ],
    )
}

fn is_follow_through_trigger(value: &str) -> bool {
    contains_any(
        value,
        &[
            "follow through trigger",
            "follow-through trigger",
            "follow through start",
            "follow-through start",
            "follow through source",
            "후속 확인 시작",
            "후속 확인 트리거",
        ],
    )
}

fn is_follow_through_result(value: &str) -> bool {
    contains_any(
        value,
        &[
            "follow through result",
            "follow-through result",
            "follow through response",
            "follow-through response",
            "follow through destination",
            "후속 확인 결과",
            "후속 확인 응답",
        ],
    )
}

fn is_cross_region_conflict_intent(value: &str) -> bool {
    contains_any(
        value,
        &[
            "watched regions show opposing",
            "regions show opposing",
            "regions disagree",
            "cross check regions",
            "cross-check regions",
            "cross region conflict",
            "cross-region conflict",
            "영역 상태 불일치",
            "영역이 서로 모순",
            "영역 상태가 서로 모순",
            "영역 상태가 충돌",
        ],
    )
}

fn numeric_rule(value: &str) -> Option<(NumberOperator, f64)> {
    let operator = if contains_any(value, &["at least", "or more", "이상", "도달"]) {
        NumberOperator::AtLeast
    } else if contains_any(
        value,
        &["greater than", "above", "over", "exceeds", "초과", "넘"],
    ) {
        NumberOperator::GreaterThan
    } else if contains_any(value, &["at most", "or less", "이하"]) {
        NumberOperator::AtMost
    } else if contains_any(value, &["less than", "below", "under", "미만", "아래"]) {
        NumberOperator::LessThan
    } else if is_progress_intent(value) && contains_any(value, &["reach", "complete", "완료"]) {
        NumberOperator::AtLeast
    } else {
        return None;
    };
    extract_numbers(value)
        .last()
        .copied()
        .map(|number| (operator, number))
}

fn state_rule(value: &str) -> Option<WatchState> {
    if contains_any(
        value,
        &[
            "error", "failed", "failure", "fails", "오류", "에러", "실패",
        ],
    ) {
        Some(WatchState::Error)
    } else if contains_any(value, &["warning", "warn", "경고", "주의"]) {
        Some(WatchState::Warning)
    } else if contains_any(
        value,
        &[
            "success",
            "succeeds",
            "passed",
            "complete",
            "completed",
            "성공",
            "완료",
        ],
    ) {
        Some(WatchState::Success)
    } else {
        None
    }
}

fn text_rule(original: &str, normalized: &str, appears: bool) -> Option<String> {
    let markers: &[&str] = if appears {
        &[
            " appears",
            " appear",
            " is shown",
            " shows up",
            "나타나",
            "표시되",
            "보이면",
        ]
    } else {
        &[
            " disappears",
            " disappear",
            " is removed",
            "사라지",
            "없어지",
        ]
    };
    if !contains_any(normalized, markers) {
        return None;
    }
    if let Some(quoted) = quoted_text(original) {
        return normalized_target(&quoted);
    }
    let prefix_stripped = strip_intent_prefix(normalized);
    let marker_index = markers
        .iter()
        .filter_map(|marker| prefix_stripped.find(marker))
        .min()?;
    normalized_target(&prefix_stripped[..marker_index])
}

fn strip_intent_prefix(value: &str) -> &str {
    [
        "tell me when ",
        "notify me when ",
        "alert me when ",
        "let me know when ",
        "watch for ",
    ]
    .into_iter()
    .find_map(|prefix| value.strip_prefix(prefix))
    .unwrap_or(value)
}

fn normalized_target(value: &str) -> Option<String> {
    let value = value
        .trim_matches(|character: char| {
            character.is_whitespace() || "'\".,:;!?".contains(character)
        })
        .trim_end_matches(['이', '가', '을', '를']);
    (!value.is_empty() && value.chars().count() <= 120).then(|| normalize(value))
}

fn quoted_text(value: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let Some(start) = value.find(quote) else {
            continue;
        };
        let rest = &value[start + quote.len_utf8()..];
        let Some(end) = rest.find(quote) else {
            continue;
        };
        let candidate = rest[..end].trim();
        if !candidate.is_empty() {
            return Some(candidate.to_string());
        }
    }
    None
}

pub(super) fn extract_numbers(value: &str) -> Vec<f64> {
    value
        .split(|character: char| {
            !character.is_ascii_digit() && !matches!(character, '.' | '-' | ',')
        })
        .filter_map(|token| token.replace(',', "").parse::<f64>().ok())
        .filter(|number| number.is_finite())
        .collect()
}

fn is_text_change_intent(value: &str) -> bool {
    contains_any(
        value,
        &[
            "text changes",
            "value changes",
            "내용이 바뀌",
            "텍스트가 바뀌",
            "값이 바뀌",
        ],
    )
}

fn is_stuck_intent(value: &str) -> bool {
    contains_any(
        value,
        &[
            "stops changing",
            "stop changing",
            "stopped changing",
            "no longer changes",
            "gets stuck",
            "get stuck",
            "is stuck",
            "stalls",
            "stalled",
            "no progress",
            "progress stops",
            "진행이 멈",
            "변화가 멈",
            "변화가 없",
            "바뀌지 않",
            "멈추면",
            "멈췄",
            "정체",
        ],
    )
}

fn is_progress_intent(value: &str) -> bool {
    contains_any(value, &["progress", "percent", "%", "진행률", "퍼센트"])
}
