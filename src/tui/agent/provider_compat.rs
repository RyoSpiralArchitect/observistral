use super::*;

fn mistral_name_embedded_blocks_and_tool(
    tc: &ToolCallData,
) -> Option<(Vec<String>, Option<ToolCallData>)> {
    let raw = tc.name.trim();
    let low = raw.to_ascii_lowercase();
    if !(low.starts_with("plan>")
        || low.contains("<plan>")
        || low.contains("<think>")
        || low.contains("</plan>")
        || low.contains("</think>"))
    {
        return None;
    }

    let mut blocks = Vec::new();
    let mut rest = raw;
    let mut tool_from_think: Option<String> = None;

    if low.starts_with("plan>")
        && !low.contains("<think>")
        && !low.contains("<goal>")
        && !low.contains("<steps>")
        && !low.contains("<acceptance")
        && !low.contains("<risks>")
        && !low.contains("<assumptions>")
    {
        if let Some(block) = inline_block_from_name(raw) {
            if parse_plan_block(&block).is_some() {
                blocks.push(block);
                if let Some(plan_end) = low.find("</plan>") {
                    rest = &raw[plan_end + "</plan>".len()..];
                } else {
                    rest = "";
                }
            }
        }
    }

    if blocks.is_empty() {
        if let Some(after_plan_prefix) = raw.strip_prefix("plan>") {
            let after_low = after_plan_prefix.to_ascii_lowercase();
            if let Some(end) = after_low.find("</plan>") {
                let plan_chunk = &after_plan_prefix[..end + "</plan>".len()];
                let block = format!("<plan>{plan_chunk}");
                if parse_plan_block(&block).is_some() {
                    blocks.push(block);
                    rest = &after_plan_prefix[end + "</plan>".len()..];
                }
            }
        }
    }

    if blocks.is_empty() {
        if let Some(plan_body) = extract_tag_block(raw, "plan") {
            let block = format!("<plan>\n{plan_body}\n</plan>");
            if parse_plan_block(&block).is_some() {
                blocks.push(block);
                if let Some(plan_end) = low.find("</plan>") {
                    rest = &raw[plan_end + "</plan>".len()..];
                }
            }
        }
    }

    let think_search = if rest == raw { raw } else { rest };
    if let Some(think_body) = extract_tag_block(think_search, "think") {
        let block = format!("<think>\n{think_body}\n</think>");
        if let Some(think) = parse_think_block(&block) {
            if !think.tool.trim().is_empty() {
                tool_from_think = Some(think.tool.clone());
            }
            blocks.push(block);
            if let Some(think_end) = think_search.to_ascii_lowercase().find("</think>") {
                rest = &think_search[think_end + "</think>".len()..];
            }
        }
    }

    if blocks.is_empty() {
        return None;
    }

    if let Some((mut prelude, normalized)) = mistral_nested_tool_payload(tc) {
        blocks.append(&mut prelude);
        return Some((blocks, Some(normalized)));
    }

    let normalized = tool_from_think
        .map(|tool| ToolCallData {
            id: tc.id.clone(),
            name: tool,
            arguments: tc.arguments.clone(),
        })
        .or_else(|| {
            known_runtime_tool_name_from_text(rest).map(|tool| ToolCallData {
                id: tc.id.clone(),
                name: tool,
                arguments: tc.arguments.clone(),
            })
        });

    Some((blocks, normalized))
}

fn inline_block_from_name(name: &str) -> Option<String> {
    let trimmed = name.trim();
    let (tag, rest, fields, close_tag) = if let Some(rest) = trimmed.strip_prefix("plan>") {
        (
            "plan",
            rest.trim(),
            [
                "goal:",
                "steps:",
                "acceptance:",
                "acceptance_criteria:",
                "risks:",
                "assumptions:",
            ]
            .as_slice(),
            "</plan>",
        )
    } else if let Some(rest) = trimmed.strip_prefix("think>") {
        (
            "think",
            rest.trim(),
            [
                "goal:", "step:", "tool:", "risk:", "doubt:", "next:", "verify:",
            ]
            .as_slice(),
            "</think>",
        )
    } else {
        return None;
    };

    let mut body = rest.to_string();
    for field in fields.iter().skip(1) {
        body = body.replace(field, &format!("\n{field}"));
    }
    body = body.replace(close_tag, &format!("\n{close_tag}"));
    let body = body.trim();
    if body.is_empty() {
        return None;
    }
    Some(format!("<{tag}>\n{body}"))
}

fn markdownish_block_fields(raw: &str) -> Option<String> {
    let mut body = raw
        .replace("**•**", "\n")
        .replace('•', "\n")
        .replace("**", "")
        .replace('`', "")
        .replace("</thinking>", "")
        .replace("</plan>", "")
        .replace("</think>", "");
    body = compact_multiline_block(body.as_str());
    let body = body.trim();
    if body.is_empty() {
        None
    } else {
        Some(body.to_string())
    }
}

fn markdownish_mistral_blocks_from_name(name: &str) -> Vec<String> {
    let trimmed = name.trim();
    let low = trimmed.to_ascii_lowercase();
    if !low.starts_with("plan") {
        return Vec::new();
    }
    let looks_markdownish = trimmed.contains("**")
        || trimmed.contains('•')
        || low.contains("**goal")
        || low.contains("**steps")
        || low.contains("**acceptance")
        || low.contains("<thinking>");
    if !looks_markdownish {
        return Vec::new();
    }

    let plan_raw = if let Some((plan, _think)) = trimmed.split_once("<thinking>") {
        plan
    } else {
        trimmed
    };
    let mut blocks = Vec::new();
    let mut plan_body = plan_raw.trim().to_string();
    if plan_body.to_ascii_lowercase().starts_with("plan") {
        plan_body = plan_body[4..]
            .trim_start_matches(|c: char| c == ':' || c == '>' || c.is_whitespace())
            .to_string();
    }
    if let Some(body) = markdownish_block_fields(plan_body.as_str()) {
        if parse_plan_block(format!("<plan>\n{body}\n</plan>").as_str()).is_some() {
            blocks.push(format!("<plan>\n{body}\n</plan>"));
        }
    }

    blocks
}

fn mistral_nested_tool_payload(tc: &ToolCallData) -> Option<(Vec<String>, ToolCallData)> {
    let value: serde_json::Value = serde_json::from_str(&tc.arguments).ok()?;
    let obj = value.as_object()?;
    let mut prelude = Vec::new();
    if let Some(think) = obj.get("think").and_then(|v| v.as_str()) {
        let think = think.trim();
        if !think.is_empty() {
            prelude.push(format!("<think>\n{think}\n</think>"));
        }
    }
    let tool_name = obj
        .get("tool")
        .and_then(|v| v.as_str())
        .or_else(|| {
            obj.get("function")
                .and_then(|v| v.get("name"))
                .and_then(|v| v.as_str())
        })?
        .trim();
    let tool_args = obj
        .get("arguments")
        .or_else(|| obj.get("function").and_then(|v| v.get("parameters")))?;
    if tool_name.is_empty() {
        return None;
    }
    Some((
        prelude,
        ToolCallData {
            id: tc.id.clone(),
            name: tool_name.to_string(),
            arguments: tool_args.to_string(),
        },
    ))
}

fn split_concatenated_json_values(raw: &str) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    let de = serde_json::Deserializer::from_str(raw).into_iter::<serde_json::Value>();
    for value in de {
        match value {
            Ok(v) => out.push(v),
            Err(_) => return Vec::new(),
        }
    }
    out
}

pub(super) fn normalize_mistral_tool_call(
    tc: &ToolCallData,
) -> (Vec<String>, Option<ToolCallData>) {
    if let Some(block) = pseudo_tool_call_to_block_text(tc) {
        return (vec![block], None);
    }
    if let Some((blocks, normalized)) = mistral_name_embedded_blocks_and_tool(tc) {
        return (blocks, normalized);
    }
    let markdownish_blocks = markdownish_mistral_blocks_from_name(&tc.name);
    if !markdownish_blocks.is_empty() {
        if let Some((mut prelude, normalized)) = mistral_nested_tool_payload(tc) {
            let mut blocks = markdownish_blocks;
            blocks.append(&mut prelude);
            return (blocks, Some(normalized));
        }
        return (markdownish_blocks, None);
    }
    if let Some(block) = inline_block_from_name(&tc.name) {
        if let Some((mut prelude, normalized)) = mistral_nested_tool_payload(tc) {
            prelude.insert(0, block);
            return (prelude, Some(normalized));
        }
        return (vec![block], None);
    }

    let mut prelude = Vec::new();
    let values = split_concatenated_json_values(&tc.arguments);
    if values.len() > 1 {
        for value in values.iter().take(values.len().saturating_sub(1)) {
            if let Some(block) = pseudo_block_text_from_named_args("plan", value)
                .or_else(|| pseudo_block_text_from_named_args("think", value))
            {
                prelude.push(block);
            }
        }

        if let Some(last) = values.last() {
            return (
                prelude,
                Some(ToolCallData {
                    id: tc.id.clone(),
                    name: tc.name.trim_matches(|c| c == '<' || c == '>').to_string(),
                    arguments: last.to_string(),
                }),
            );
        }
    }

    (
        prelude,
        Some(ToolCallData {
            id: tc.id.clone(),
            name: tc.name.trim_matches(|c| c == '<' || c == '>').to_string(),
            arguments: tc.arguments.clone(),
        }),
    )
}

pub(super) fn build_mistral_think_only_hint(root_user_text: &str, think: &ThinkBlock) -> String {
    let pattern = preferred_read_only_search_pattern(root_user_text);
    let dir = preferred_read_only_search_dir(root_user_text);
    let read_path = preferred_read_only_read_path_hint(root_user_text);
    let tool = think.tool.trim();
    let suggested = match tool {
        "search_files" => format!("search_files(pattern=\"{pattern}\", dir=\"{dir}\")"),
        "list_dir" => format!("list_dir(dir=\"{dir}\")"),
        "glob" => format!("glob(pattern=\"*{pattern}*\", dir=\"{dir}\")"),
        "read_file" => format!("read_file(path=\"{read_path}\")"),
        _ => format!("search_files(pattern=\"{pattern}\", dir=\"{dir}\")"),
    };

    format!(
        "[Mistral compatibility]\n\
`think` is not a real tool call.\n\
Next assistant turn:\n\
1) emit the <think> block as plain text\n\
2) call ONE real tool immediately after it\n\
Suggested real tool for this task: {suggested}\n\
Do NOT emit a `think` tool_call again."
    )
}

pub(super) fn mistral_observation_tool(name: &str) -> bool {
    matches!(
        name,
        "read_file" | "search_files" | "list_dir" | "glob" | "done"
    )
}

pub(super) fn rescue_read_only_missing_plan_for_tool_turn(
    messages: &[serde_json::Value],
    tc: &ToolCallData,
    root_user_text: &str,
    root_read_only: bool,
    provider: ProviderKind,
) -> Option<PlanBlock> {
    if !root_read_only || !mistral_observation_tool(tc.name.as_str()) {
        return None;
    }
    let prior_blocks = consecutive_missing_plan_blocks_for_tool(messages, tc);
    let prior_observation_blocks = consecutive_missing_plan_blocks_for_observation(messages);
    if prior_blocks.saturating_add(1) < 2 && prior_observation_blocks.saturating_add(1) < 3 {
        return None;
    }
    if matches!(provider, ProviderKind::Mistral) {
        if let Some(repaired) =
            repair_mistral_plan_for_tool_turn(None, None, tc, root_user_text, root_read_only)
        {
            return Some(repaired);
        }
    }
    Some(synthetic_read_only_observation_plan(root_user_text))
}

pub(super) fn rescue_read_only_missing_think_for_tool_turn(
    messages: &[serde_json::Value],
    tc: &ToolCallData,
    plan: &PlanBlock,
    root_user_text: &str,
    root_read_only: bool,
    provider: ProviderKind,
    evidence: &ObservationEvidence,
) -> Option<ThinkBlock> {
    if !root_read_only || !mistral_observation_tool(tc.name.as_str()) {
        return None;
    }
    if tc.name == "read_file" {
        return Some(compat_synthetic_think(tc, plan));
    }
    if tc.name == "done"
        && build_read_only_completion_hint(
            root_user_text,
            plan,
            evidence,
            messages,
            &WorkingMemory::default(),
        )
        .is_some()
    {
        return Some(compat_synthetic_think(tc, plan));
    }
    if matches!(provider, ProviderKind::Mistral) {
        return Some(compat_synthetic_think(tc, plan));
    }
    let prior_blocks = consecutive_missing_think_blocks_for_tool(messages, tc);
    let prior_observation_blocks = consecutive_missing_think_blocks_for_observation(messages);
    if prior_blocks.saturating_add(1) < 2 && prior_observation_blocks.saturating_add(1) < 2 {
        return None;
    }
    Some(compat_synthetic_think(tc, plan))
}

pub(super) fn repair_mistral_plan_for_tool_turn(
    parsed_plan: Option<&PlanBlock>,
    active_plan: Option<&PlanBlock>,
    tc: &ToolCallData,
    root_user_text: &str,
    root_read_only: bool,
) -> Option<PlanBlock> {
    if !root_read_only || !mistral_observation_tool(tc.name.as_str()) {
        return None;
    }
    let fallback = synthetic_read_only_observation_plan(root_user_text);
    let mut repaired = active_plan.cloned().unwrap_or_else(|| fallback.clone());
    if let Some(parsed) = parsed_plan {
        if !parsed.goal.trim().is_empty() {
            repaired.goal = parsed.goal.clone();
        }
        if !parsed.steps.is_empty() {
            repaired.steps = parsed.steps.clone();
        }
        if !parsed.acceptance_criteria.is_empty() {
            repaired.acceptance_criteria = parsed.acceptance_criteria.clone();
        }
        if !parsed.risks.trim().is_empty() {
            repaired.risks = parsed.risks.clone();
        }
        if !parsed.assumptions.trim().is_empty() {
            repaired.assumptions = parsed.assumptions.clone();
        }
    }
    if repaired.steps.len() < plan_field_min_items("steps").unwrap_or(2) {
        repaired.steps = fallback.steps;
    }
    if repaired.acceptance_criteria.len() < plan_field_min_items("acceptance").unwrap_or(1) {
        repaired.acceptance_criteria = fallback.acceptance_criteria;
    }
    if repaired.risks.trim().is_empty() {
        repaired.risks = fallback.risks;
    }
    if repaired.assumptions.trim().is_empty() {
        repaired.assumptions = fallback.assumptions;
    }
    Some(repaired)
}

pub(super) fn compat_synthetic_think(tc: &ToolCallData, plan: &PlanBlock) -> ThinkBlock {
    let next = match tc.name.as_str() {
        "exec" => parse_exec_command_from_args(&tc.arguments)
            .unwrap_or_else(|| "run the command".to_string()),
        "read_file" => serde_json::from_str::<serde_json::Value>(&tc.arguments)
            .ok()
            .and_then(|v| {
                v.get("path")
                    .and_then(|x| x.as_str())
                    .map(|path| format!("read {path}"))
            })
            .unwrap_or_else(|| "read the target file".to_string()),
        "search_files" => serde_json::from_str::<serde_json::Value>(&tc.arguments)
            .ok()
            .and_then(|v| {
                let pattern = v
                    .get("pattern")
                    .and_then(|x| x.as_str())
                    .unwrap_or("pattern");
                let dir = v.get("dir").and_then(|x| x.as_str()).unwrap_or(".");
                Some(format!("search {pattern} in {dir}"))
            })
            .unwrap_or_else(|| "search the codebase".to_string()),
        "list_dir" => serde_json::from_str::<serde_json::Value>(&tc.arguments)
            .ok()
            .and_then(|v| {
                v.get("dir")
                    .and_then(|x| x.as_str())
                    .map(|dir| format!("list {dir}"))
            })
            .unwrap_or_else(|| "list the directory".to_string()),
        "glob" => serde_json::from_str::<serde_json::Value>(&tc.arguments)
            .ok()
            .and_then(|v| {
                let pattern = v.get("pattern").and_then(|x| x.as_str()).unwrap_or("*");
                let dir = v.get("dir").and_then(|x| x.as_str()).unwrap_or(".");
                Some(format!("glob {pattern} in {dir}"))
            })
            .unwrap_or_else(|| "glob for candidate files".to_string()),
        _ => format!("run {}", tc.name),
    };

    ThinkBlock {
        goal: compact_one_line(plan.goal.as_str(), 120),
        step: 1,
        tool: tc.name.clone(),
        risk: "wrong target or overly broad action".to_string(),
        doubt: "may need one narrower follow-up".to_string(),
        next,
        verify: "confirm the tool output matches the request".to_string(),
    }
}

pub(super) fn select_think_for_tool_turn<'a>(
    parsed: Option<&'a ThinkBlock>,
    compat_last: Option<&'a ThinkBlock>,
    compat_synth: Option<&'a ThinkBlock>,
    plan: &PlanBlock,
    tc: &ToolCallData,
    provider: ProviderKind,
) -> (Option<&'a ThinkBlock>, bool) {
    if matches!(provider, ProviderKind::Mistral) {
        if let Some(think) = parsed.filter(|think| validate_think(think, plan, tc).is_ok()) {
            return (Some(think), false);
        }
        if let Some(think) = compat_synth.filter(|think| validate_think(think, plan, tc).is_ok()) {
            return (Some(think), true);
        }
        if let Some(think) = compat_last.filter(|think| validate_think(think, plan, tc).is_ok()) {
            return (Some(think), false);
        }
        return (
            parsed.or(compat_synth).or(compat_last),
            compat_synth.is_some(),
        );
    }

    (parsed.or(compat_last).or(compat_synth), false)
}
