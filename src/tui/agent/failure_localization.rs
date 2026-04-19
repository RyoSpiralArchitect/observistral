use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default, Clone)]
struct FailureContext {
    failing_tests: BTreeSet<String>,
    failure_tokens: BTreeSet<String>,
    mismatch_tokens: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct TestContext {
    name: String,
    name_tokens: BTreeSet<String>,
    calls: Vec<String>,
    assertion_tokens: BTreeSet<String>,
    expected_transition: Option<(String, String)>,
}

pub(super) fn interesting_failure_line(stdout: &str, stderr: &str) -> String {
    for src in [stderr, stdout] {
        if let Some(line) = preferred_failure_line(src) {
            return normalize_line(line);
        }
    }

    let keywords = [
        "error",
        "fatal",
        "exception",
        "traceback",
        "parsererror",
        "unexpected token",
        "not recognized",
        "commandnotfoundexception",
        "missing expression",
        "unable to",
        "could not",
        "access is denied",
        "permission denied",
    ];

    for src in [stderr, stdout] {
        for line in src.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let low = trimmed.to_ascii_lowercase();
            if keywords.iter().any(|keyword| low.contains(keyword)) {
                return normalize_line(trimmed);
            }
        }
    }

    for src in [stderr, stdout] {
        for line in src.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                return normalize_line(trimmed);
            }
        }
    }

    String::new()
}

pub(super) fn infer_fix_existing_symbol(
    messages: &[Value],
    reads: &BTreeMap<String, String>,
    impl_path: &str,
) -> Option<String> {
    recent_attempted_patch_symbol(messages, impl_path)
        .or_else(|| infer_test_localized_symbol(messages, reads, impl_path))
}

fn preferred_failure_line(text: &str) -> Option<&str> {
    let mut in_failures = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            in_failures = false;
            continue;
        }

        if trimmed.starts_with("thread '") && trimmed.contains("' panicked at ") {
            return Some(trimmed);
        }
        if trimmed.starts_with("---- ") && trimmed.ends_with(" stdout ----") {
            return Some(trimmed);
        }
        if trimmed.starts_with("test ") && trimmed.ends_with(" ... FAILED") {
            return Some(trimmed);
        }
        if trimmed.contains("assertion") && trimmed.contains("failed") {
            return Some(trimmed);
        }

        if trimmed == "failures:" {
            in_failures = true;
            continue;
        }
        if in_failures && !trimmed.starts_with("failures:") {
            return Some(trimmed);
        }
    }

    None
}

fn recent_attempted_patch_symbol(messages: &[Value], impl_path: &str) -> Option<String> {
    for msg in messages.iter().rev() {
        let Some(tool_calls) = msg.get("tool_calls").and_then(|value| value.as_array()) else {
            continue;
        };
        for tool_call in tool_calls.iter().rev() {
            let Some(function) = tool_call.get("function") else {
                continue;
            };
            let name = function
                .get("name")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .trim();
            if name != "patch_file" {
                continue;
            }
            let arguments = function
                .get("arguments")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .trim();
            let Ok(parsed) = serde_json::from_str::<Value>(arguments) else {
                continue;
            };
            let path = parsed
                .get("path")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .trim();
            if path != impl_path {
                continue;
            }
            let search = parsed
                .get("search")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let replace = parsed
                .get("replace")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            if let Some(symbol) = extract_function_name_from_patch_text(search)
                .or_else(|| extract_function_name_from_patch_text(replace))
            {
                return Some(symbol);
            }
        }
    }
    None
}

fn infer_test_localized_symbol(
    messages: &[Value],
    reads: &BTreeMap<String, String>,
    impl_path: &str,
) -> Option<String> {
    let impl_body = reads.get(impl_path)?;
    let impl_functions = impl_function_names(impl_body);
    if impl_functions.is_empty() {
        return None;
    }

    let tests = collect_test_contexts(reads, impl_path);
    if tests.is_empty() {
        return None;
    }

    let failure = collect_failure_context(messages);
    let mut best: Option<(i32, String)> = None;

    for test in tests {
        let test_match = failure.failing_tests.iter().any(|candidate| {
            candidate == &test.name
                || candidate.ends_with(test.name.as_str())
                || test.name.ends_with(candidate.as_str())
        });
        let mismatch_overlap = intersection_len(&test.assertion_tokens, &failure.mismatch_tokens);
        let failure_overlap = intersection_len(&test.name_tokens, &failure.failure_tokens);

        for call in &test.calls {
            if !impl_functions.contains(call.as_str()) {
                continue;
            }
            let mut score = 1i32;
            score += semantic_overlap(call, &test.name_tokens) * 4;
            score += mismatch_overlap * 8;
            score += failure_overlap * 4;
            if test_match {
                score += 80;
            }
            if let Some((source, expected)) = test.expected_transition.as_ref() {
                if let Some(actual) = actual_transition_for_function(impl_body, call, source) {
                    if actual == *expected {
                        score -= 6;
                    } else {
                        score += 40;
                    }
                }
            }

            match &best {
                Some((best_score, best_name))
                    if score < *best_score || (score == *best_score && call >= best_name) => {}
                _ => best = Some((score, call.clone())),
            }
        }
    }

    best.and_then(|(score, symbol)| (score > 0).then_some(symbol))
}

fn collect_failure_context(messages: &[Value]) -> FailureContext {
    let mut out = FailureContext::default();
    let mut in_failures = false;

    for msg in messages {
        let Some(content) = msg.get("content").and_then(|value| value.as_str()) else {
            continue;
        };

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                in_failures = false;
                continue;
            }

            if trimmed == "failures:" {
                in_failures = true;
                continue;
            }
            if in_failures {
                out.failing_tests.insert(trimmed.to_string());
                out.failure_tokens.extend(normalized_tokens(trimmed));
                continue;
            }

            if let Some(test_name) = parse_thread_panicked_test(trimmed)
                .or_else(|| parse_stdout_header_test(trimmed))
                .or_else(|| parse_failed_test_line(trimmed))
            {
                out.failure_tokens
                    .extend(normalized_tokens(test_name.as_str()));
                out.failing_tests.insert(test_name);
                continue;
            }

            if let Some(value) = trimmed.strip_prefix("left: ") {
                out.mismatch_tokens.extend(normalized_tokens(value));
                continue;
            }
            if let Some(value) = trimmed.strip_prefix("right: ") {
                out.mismatch_tokens.extend(normalized_tokens(value));
                continue;
            }

            if trimmed.contains("assertion") && trimmed.contains("failed") {
                out.failure_tokens.extend(normalized_tokens(trimmed));
            }
        }
    }

    out
}

fn collect_test_contexts(reads: &BTreeMap<String, String>, impl_path: &str) -> Vec<TestContext> {
    let mut out = Vec::new();
    for (path, body) in reads {
        if path == impl_path || !body.contains("#[test]") {
            continue;
        }
        out.extend(parse_test_contexts(body));
    }
    out
}

fn parse_test_contexts(body: &str) -> Vec<TestContext> {
    let mut out = Vec::new();
    let mut pending_test = false;
    let mut current_name: Option<String> = None;
    let mut current_lines: Vec<String> = Vec::new();
    let mut brace_depth = 0i32;
    let mut saw_open_brace = false;

    for line in body.lines() {
        let trimmed = line.trim();
        if current_name.is_none() {
            if trimmed.starts_with("#[test]") {
                pending_test = true;
                continue;
            }
            if pending_test {
                if let Some(name) = parse_function_name(trimmed) {
                    current_name = Some(name);
                    current_lines.clear();
                    current_lines.push(line.to_string());
                    let delta = brace_delta(line);
                    brace_depth = delta;
                    saw_open_brace = line.contains('{');
                    pending_test = false;
                    if saw_open_brace && brace_depth <= 0 {
                        finalize_test_context(&mut out, &mut current_name, &mut current_lines);
                        brace_depth = 0;
                        saw_open_brace = false;
                    }
                }
            }
            continue;
        }

        current_lines.push(line.to_string());
        let delta = brace_delta(line);
        if line.contains('{') {
            saw_open_brace = true;
        }
        brace_depth += delta;
        if saw_open_brace && brace_depth <= 0 {
            finalize_test_context(&mut out, &mut current_name, &mut current_lines);
            brace_depth = 0;
            saw_open_brace = false;
        }
    }

    out
}

fn finalize_test_context(
    out: &mut Vec<TestContext>,
    current_name: &mut Option<String>,
    current_lines: &mut Vec<String>,
) {
    let Some(name) = current_name.take() else {
        return;
    };
    let body = current_lines.join("\n");
    let calls = test_call_candidates(body.as_str());
    let assertion_tokens = assertion_tokens(body.as_str());
    out.push(TestContext {
        name: name.clone(),
        name_tokens: normalized_tokens(name.as_str()),
        calls,
        assertion_tokens,
        expected_transition: expected_transition(body.as_str(), name.as_str()),
    });
    current_lines.clear();
}

fn parse_function_name(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix("fn ")?;
    let (name, _) = rest.split_once('(')?;
    let name = name.trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn brace_delta(line: &str) -> i32 {
    let opens = line.chars().filter(|ch| *ch == '{').count() as i32;
    let closes = line.chars().filter(|ch| *ch == '}').count() as i32;
    opens - closes
}

fn test_call_candidates(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = body.as_bytes();
    let mut idx = 0usize;
    while idx < bytes.len() {
        let ch = bytes[idx] as char;
        if !(ch.is_ascii_alphabetic() || ch == '_') {
            idx += 1;
            continue;
        }
        let start = idx;
        idx += 1;
        while idx < bytes.len() {
            let ch = bytes[idx] as char;
            if ch.is_ascii_alphanumeric() || ch == '_' {
                idx += 1;
            } else {
                break;
            }
        }
        let ident = &body[start..idx];
        let mut lookahead = idx;
        while lookahead < bytes.len() && (bytes[lookahead] as char).is_ascii_whitespace() {
            lookahead += 1;
        }
        if lookahead >= bytes.len() || bytes[lookahead] as char != '(' {
            continue;
        }
        if is_ignored_call_candidate(ident) {
            continue;
        }
        if start >= 3 && &body[start.saturating_sub(3)..start] == "fn " {
            continue;
        }
        if !out.iter().any(|existing| existing == ident) {
            out.push(ident.to_string());
        }
    }
    out
}

fn is_ignored_call_candidate(ident: &str) -> bool {
    matches!(
        ident,
        "assert" | "assert_eq" | "assert_ne" | "matches" | "println" | "format"
    )
}

fn assertion_tokens(body: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("assert") {
            continue;
        }
        out.extend(normalized_tokens(trimmed));
    }
    out
}

fn expected_transition(body: &str, test_name: &str) -> Option<(String, String)> {
    parse_transition_from_test_name(test_name).or_else(|| parse_transition_from_assert(body))
}

fn parse_transition_from_test_name(test_name: &str) -> Option<(String, String)> {
    let tokens = split_raw_tokens(test_name);
    for window in tokens.windows(4) {
        if matches!(window, [from, src, points, dst] if from == "from" && points == "points") {
            return Some((
                normalize_token(window[1].as_str()),
                normalize_token(window[3].as_str()),
            ));
        }
    }
    None
}

fn parse_transition_from_assert(body: &str) -> Option<(String, String)> {
    for line in body.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("assert_eq!") {
            continue;
        }
        let variants = enum_variants(trimmed);
        if variants.len() >= 2 {
            return Some((variants[0].clone(), variants[1].clone()));
        }
    }
    None
}

fn enum_variants(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut idx = 0usize;
    while idx < bytes.len() {
        let ch = bytes[idx] as char;
        if !(ch.is_ascii_alphabetic() || ch == '_') {
            idx += 1;
            continue;
        }
        let start = idx;
        idx += 1;
        while idx < bytes.len() {
            let ch = bytes[idx] as char;
            if ch.is_ascii_alphanumeric() || ch == '_' {
                idx += 1;
            } else {
                break;
            }
        }
        let first = &text[start..idx];
        let mut probe = idx;
        while probe + 1 < bytes.len() && (bytes[probe] as char).is_ascii_whitespace() {
            probe += 1;
        }
        if probe + 1 >= bytes.len() || &text[probe..probe + 2] != "::" {
            continue;
        }
        probe += 2;
        while probe < bytes.len() && (bytes[probe] as char).is_ascii_whitespace() {
            probe += 1;
        }
        let second_start = probe;
        while probe < bytes.len() {
            let ch = bytes[probe] as char;
            if ch.is_ascii_alphanumeric() || ch == '_' {
                probe += 1;
            } else {
                break;
            }
        }
        if second_start == probe || first.is_empty() {
            idx += 1;
            continue;
        }
        let second = &text[second_start..probe];
        out.push(normalize_token(second));
        idx = probe;
    }
    out
}

fn actual_transition_for_function(
    impl_body: &str,
    function_name: &str,
    source: &str,
) -> Option<String> {
    let function_body = extract_function_body(impl_body, function_name)?;
    for line in function_body.lines() {
        let trimmed = line.trim();
        let Some((lhs, rhs)) = trimmed.split_once("=>") else {
            continue;
        };
        let lhs_variants = enum_variants(lhs);
        let rhs_variants = enum_variants(rhs);
        let Some(lhs_variant) = lhs_variants.first() else {
            continue;
        };
        let Some(rhs_variant) = rhs_variants.first() else {
            continue;
        };
        if lhs_variant == source {
            return Some(rhs_variant.clone());
        }
    }
    None
}

fn extract_function_body(body: &str, function_name: &str) -> Option<String> {
    let needle_pub = format!("pub fn {function_name}(");
    let needle_plain = format!("fn {function_name}(");
    let start = body
        .find(needle_pub.as_str())
        .or_else(|| body.find(needle_plain.as_str()))?;
    let tail = &body[start..];
    let mut brace_depth = 0i32;
    let mut saw_open = false;
    let mut out = String::new();
    for ch in tail.chars() {
        out.push(ch);
        if ch == '{' {
            brace_depth += 1;
            saw_open = true;
        } else if ch == '}' {
            brace_depth -= 1;
            if saw_open && brace_depth <= 0 {
                break;
            }
        }
    }
    Some(out)
}

fn impl_function_names(body: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in body.lines() {
        let trimmed = line.trim();
        let rest = trimmed
            .strip_prefix("pub fn ")
            .or_else(|| trimmed.strip_prefix("fn "));
        let Some(rest) = rest else {
            continue;
        };
        let Some((name, _)) = rest.split_once('(') else {
            continue;
        };
        let name = name.trim();
        if !name.is_empty()
            && name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            out.insert(name.to_string());
        }
    }
    out
}

fn parse_thread_panicked_test(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix("thread '")?;
    let (name, _) = rest.split_once("' panicked at ")?;
    if name.trim().is_empty() {
        None
    } else {
        Some(name.trim().to_string())
    }
}

fn parse_stdout_header_test(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix("---- ")?;
    let (name, _) = rest.split_once(" stdout ----")?;
    if name.trim().is_empty() {
        None
    } else {
        Some(name.trim().to_string())
    }
}

fn parse_failed_test_line(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix("test ")?;
    let (name, _) = rest.split_once(" ... FAILED")?;
    if name.trim().is_empty() {
        None
    } else {
        Some(name.trim().to_string())
    }
}

fn split_raw_tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            current.push(ch);
        } else if !current.is_empty() {
            out.push(current.clone());
            current.clear();
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

fn normalized_tokens(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for token in split_raw_tokens(text) {
        let normalized = normalize_token(token.as_str());
        if !normalized.is_empty() {
            out.insert(normalized);
        }
    }
    out
}

fn normalize_token(token: &str) -> String {
    let low = token
        .trim()
        .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .to_ascii_lowercase();
    let tail = low.rsplit("::").next().unwrap_or(low.as_str());
    if tail.len() > 4 && tail.ends_with("ing") {
        return tail[..tail.len() - 3].to_string();
    }
    if tail.len() > 3 && tail.ends_with('s') && !tail.ends_with("ss") {
        return tail[..tail.len() - 1].to_string();
    }
    tail.to_string()
}

fn semantic_overlap(candidate: &str, tokens: &BTreeSet<String>) -> i32 {
    normalized_tokens(candidate)
        .into_iter()
        .filter(|token| tokens.contains(token))
        .count() as i32
}

fn intersection_len(left: &BTreeSet<String>, right: &BTreeSet<String>) -> i32 {
    left.iter().filter(|token| right.contains(*token)).count() as i32
}

fn extract_function_name_from_patch_text(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        let rest = trimmed
            .strip_prefix("pub fn ")
            .or_else(|| trimmed.strip_prefix("fn "));
        let Some(rest) = rest else {
            continue;
        };
        let (name, _) = rest.split_once('(')?;
        let name = name.trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

fn normalize_line(line: &str) -> String {
    line.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn interesting_failure_line_prefers_panicked_test_name() {
        let stdout =
            "running 2 tests\ntest tests::turning_left_from_north_points_west ... FAILED\n";
        let stderr = "thread 'tests::turning_left_from_north_points_west' panicked at src/main.rs:10:9:\nassertion `left == right` failed";
        assert_eq!(
            interesting_failure_line(stdout, stderr),
            "thread 'tests::turning_left_from_north_points_west' panicked at src/main.rs:10:9:"
        );
    }

    #[test]
    fn infer_fix_existing_symbol_prefers_suspicious_transition() {
        let messages = vec![json!({
            "role": "tool",
            "content": "thread 'tests::turning_left_from_north_points_west' panicked at src/main.rs:19:9:\nassertion `left == right` failed\n  left: East\n right: West"
        })];
        let reads = BTreeMap::from([
            (
                "src/main.rs".to_string(),
                "mod robot;\n\n#[cfg(test)]\nmod tests {\n    use super::robot::{Heading, Robot};\n\n    #[test]\n    fn turning_right_from_north_points_east() {\n        let mut robot = Robot::demo();\n        robot.turn_right();\n        assert_eq!(robot.heading(), Heading::East);\n    }\n\n    #[test]\n    fn turning_left_from_north_points_west() {\n        let mut robot = Robot::demo();\n        robot.turn_left();\n        assert_eq!(robot.heading(), Heading::West);\n    }\n}\n"
                    .to_string(),
            ),
            (
                "src/robot.rs".to_string(),
                "impl Robot {\n    pub fn turn_left(&mut self) {\n        self.heading = match self.heading {\n            Heading::North => Heading::East,\n            Heading::East => Heading::North,\n            Heading::South => Heading::East,\n            Heading::West => Heading::South,\n        };\n    }\n\n    pub fn turn_right(&mut self) {\n        self.heading = match self.heading {\n            Heading::North => Heading::East,\n            Heading::East => Heading::South,\n            Heading::South => Heading::West,\n            Heading::West => Heading::North,\n        };\n    }\n}\n"
                    .to_string(),
            ),
        ]);

        assert_eq!(
            infer_fix_existing_symbol(&messages, &reads, "src/robot.rs").as_deref(),
            Some("turn_left")
        );
    }

    #[test]
    fn infer_fix_existing_symbol_uses_attempted_patch_before_generic_hint() {
        let messages = vec![json!({
            "role": "assistant",
            "tool_calls": [{
                "id": "call_patch",
                "type": "function",
                "function": {
                    "name": "patch_file",
                    "arguments": serde_json::json!({
                        "path": "src/robot.rs",
                        "search": "    pub fn turn_left(&mut self) {\n        self.heading = match self.heading {\n            Heading::North => Heading::East,\n        };\n    }\n",
                        "replace": "    pub fn turn_left(&mut self) {\n        self.heading = match self.heading {\n            Heading::North => Heading::West,\n        };\n    }\n"
                    }).to_string()
                }
            }]
        })];

        assert_eq!(
            recent_attempted_patch_symbol(&messages, "src/robot.rs").as_deref(),
            Some("turn_left")
        );
    }
}
