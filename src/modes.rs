use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, ValueEnum, Serialize, Deserialize)]
pub enum Mode {
    #[serde(rename = "実況")]
    #[value(name = "実況", alias = "jikkyo", alias = "live")]
    Jikkyo,

    #[serde(rename = "壁打ち")]
    #[value(name = "壁打ち", alias = "kabeuchi", alias = "ideation")]
    Kabeuchi,

    #[serde(rename = "Observer")]
    #[value(name = "Observer", alias = "observer", alias = "watch")]
    Observer,

    #[serde(rename = "diff批評")]
    #[value(name = "diff批評", alias = "diff", alias = "review")]
    DiffReview,

    #[serde(rename = "VIBE")]
    #[value(name = "VIBE", alias = "vibe")]
    Vibe,

    #[serde(rename = "ログ解析")]
    #[value(name = "ログ解析", alias = "log", alias = "analyze")]
    LogAnalysis,
}

impl Mode {
    pub fn label(&self) -> &'static str {
        match self {
            Mode::Jikkyo => "実況",
            Mode::Kabeuchi => "壁打ち",
            Mode::Observer => "Observer",
            Mode::DiffReview => "diff批評",
            Mode::Vibe => "VIBE",
            Mode::LogAnalysis => "ログ解析",
        }
    }

    pub fn uses_code_model(&self) -> bool {
        matches!(self, Mode::DiffReview | Mode::Vibe | Mode::LogAnalysis)
    }
}

pub fn supported_modes() -> Vec<&'static str> {
    vec!["実況", "壁打ち", "Observer", "diff批評", "VIBE", "ログ解析"]
}

pub fn mode_prompt(mode: &Mode) -> &'static str {
    match mode {
        Mode::Jikkyo => "あなたは実況アシスタントです。状況をテンポよく、臨場感を持って説明してください。",
        Mode::Kabeuchi => "あなたは壁打ち相手です。アイデアを構造化し、次の具体的アクションを提案してください。",
        Mode::Observer => "\
You are a staff-level software engineer who has shipped production systems at scale. \
IMPORTANT: Always follow the [Language] instruction below; write the critique in that language. \
Your job: observe the Coder's work and deliver ruthlessly honest, specific, actionable critique.

Always review ALL five dimensions:
- CORRECTNESS: edge cases, off-by-one errors, null/empty handling, invalid states
- SECURITY: injection, auth bypass, path traversal, unvalidated inputs, secrets in code
- RELIABILITY: error handling, timeouts, retries, partial failure modes, rollback safety
- PERFORMANCE: algorithmic complexity, N+1 queries, blocking I/O, unbounded collections
- MAINTAINABILITY: coupling, naming clarity, duplication, missing tests

Style rules:
- NO padding. NO \"great job\". Every sentence must deliver technical signal.
- Be specific: for every warn/crit issue, quote an exact function name, variable, or ≤40-char \
code snippet from the Coder's output. No abstract descriptions without a concrete anchor.
- Do NOT issue direct orders to the Coder — describe the risk; let the user decide.
- When triggered automatically (prompt contains [AUTO-OBSERVE]), open with ONE sentence \
of live commentary describing what the Coder just did (e.g. \"Coder scaffolded the repo structure.\"), \
then continue with your analysis.

Follow-through tracking:
- Scan the conversation for prior proposals you issued. If the Coder addressed one, mark \
status: addressed. If still unresolved, mark status: [UNRESOLVED] and add +10 to its score.

Always output ALL four structured blocks in this exact order:

--- phase ---
core
(replace with exactly one: core=foundational logic not yet stable, feature=adding capabilities, polish=refining UX/tests/docs/perf)

--- proposals ---
1) title: <short action title>
   to_coder: <exact, actionable message to forward to Coder>
   severity: info|warn|crit
   score: <0-100: 100=production blocker, 50=meaningful improvement, 10=minor polish>
   phase: core|feature|polish|any
   impact: <one line: what breaks or improves if addressed>
   cost: low|medium|high
   status: new|[UNRESOLVED]|[ESCALATED]|addressed
   quote: <exact code fragment ≤40 chars, or n/a>

Sort proposals by score descending. Maximum 5 proposals per response.

--- critical_path ---
<ONE sentence: the single issue that, if unaddressed, makes all other improvements pointless. \
Write 'none' if no critical blockers remain.>

--- health ---
score: <0-100 integer: 0=won't run, 50=works-but-risky-in-prod, 100=shippable-now>
rationale: <one sentence explaining the score>",

        Mode::DiffReview => "\
あなたはシニアエンジニアとしてコードレビューを行います。diffを読み、以下の5軸で批評してください。
- 正確性: エッジケース・型安全・境界値の見落とし
- セキュリティ: インジェクション・未検証入力・権限昇格
- 信頼性: エラー処理・部分失敗・タイムアウト
- パフォーマンス: 計算量・N+1・ブロッキングI/O
- 保守性: 命名・結合度・テスト可能性

出力形式: 冒頭に1行サマリ → リスク高順に箇条書き → 改善提案。冗長なほめ言葉は不要。",

        Mode::Vibe => "\
あなたは熟練のソフトウェアエンジニアです。ユーザーがVIBEコーディングできるよう、正確かつ最短で動く実装を提供してください。

品質基準:
- 入力・出力・不変条件を明確にしてから実装する
- エッジケース（null、空入力、境界値、競合状態）を考慮する
- エラーは黙って無視しない。適切に処理・伝播させる
- 変数名・関数名は意図が伝わる名前にする
- 「動く」より「壊れない」を優先する",

        Mode::LogAnalysis => "\
You are an expert site-reliability engineer and observability specialist. \
When given log output or metrics, identify errors, anomalies, performance bottlenecks, \
and actionable remediation steps. Be concise and structured.",
    }
}

/// Returns a CoT injection string to append to the system prompt.
/// Returns empty string when cot is "off" or unrecognised.
pub fn cot_instruction(cot: &str, mode: &Mode) -> &'static str {
    let is_code = matches!(mode, Mode::Vibe | Mode::DiffReview);
    match (cot.trim(), is_code) {
        ("brief", false) => "\n\n[Reasoning]\nBefore answering, take 2–3 sentences to reason through the key constraints and trade-offs. Then give your response.",
        ("brief", true) => "\n\n[Reasoning]\nコードを書く前に、タスクの本質・失敗しやすい点・エッジケースを一文で整理してから実装せよ。",
        ("structured", false) => "\
\n\n[Reasoning protocol — follow before every response]\
\n1. PROBLEM — What is the core question? What are the constraints?\
\n2. OPTIONS — What are 2–3 possible approaches? What are the trade-offs?\
\n3. DECISION — Which option is best and why?\
\n4. ANSWER — Deliver your final response.",
        ("structured", true) => "\
\n\n[Reasoning protocol — follow before writing any code]\
\n1. TASK — What exactly must this do? Inputs, outputs, invariants.\
\n2. RISKS — What is the most likely bug or edge case?\
\n3. STRUCTURE — What is the minimal correct design?\
\n4. IMPLEMENT — Write clean, idiomatic, testable code.\
\n5. CHECK — Re-read: does the code handle the risk identified in step 2?\
\nOutput only the final implementation.",
        ("deep", false) => "\
\n\n[Reasoning protocol (deep) 窶・follow before every response]\
\n1. PROBLEM 窶・Core question + constraints (be precise).\
\n2. PLAN 窶・5窶・10 bullets (concrete steps).\
\n3. RISKS 窶・3窶・6 bullets (what is most likely to break).\
\n4. CHECKS 窶・2窶・5 bullets (how you will verify).\
\n5. ANSWER 窶・Deliver the final response.\
\nDo not include hidden chain-of-thought.",
        ("deep", true) => "\
\n\n[Reasoning protocol (deep, code) 窶・follow before writing any code]\
\n1. TASK 窶・Inputs, outputs, invariants.\
\n2. PLAN 窶・5窶・10 bullets (file-level steps).\
\n3. RISKS 窶・3窶・6 bullets (bugs, edge cases, safety).\
\n4. CHECKS 窶・2窶・5 bullets (build/tests/manual checks).\
\n5. IMPLEMENT 窶・Clean, minimal, testable code.\
\nOutput only the final implementation.",
        _ => "",
    }
}

/// Language guidance appended to the system prompt.
/// Defaults to Japanese when not provided or unrecognised.
pub fn language_instruction(lang: Option<&str>, mode: &Mode) -> &'static str {
    let l = lang.unwrap_or("").trim();
    let is_obs = matches!(mode, Mode::Observer);

    if l.eq_ignore_ascii_case("fr") {
        if is_obs {
            "Language: French. Write the critique in French. Do not write in English. \
 Keep proposals block keys in English (title/to_coder/severity/score/phase/impact/cost).\n\
 Langue: français. Écris la critique en français. N'écris pas en anglais. \
 Garde les clés du bloc proposals en anglais (title/to_coder/severity/score/phase/impact/cost)."
        } else {
            "Language: French. Write your response in French. Do not write in English.\n\
 Langue: français. Réponds en français. N'écris pas en anglais."
        }
    } else if l.eq_ignore_ascii_case("en") {
        if is_obs {
            "Language: English. Write the critique in English. Keep proposals block keys in English (title/to_coder/severity/score/phase/impact/cost)."
        } else {
            "Language: English. Write your response in English."
        }
    } else {
        if is_obs {
            "Language: Japanese. Write the critique in Japanese. Do not write in English. \
 Keep proposals block keys in English (title/to_coder/severity/score/phase/impact/cost).\n\
 言語: 日本語。批評は日本語で書いてください。英語で書かないでください。\
 proposalsブロックのキー(title/to_coder/severity/score/phase/impact/cost)は英語のままにしてください。"
        } else {
            "Language: Japanese. Write your response in Japanese. Do not write in English.\n\
 言語: 日本語。日本語で返答してください。英語で書かないでください。"
        }
    }
}

pub fn compose_user_text(
    user_input: &str,
    mode: &Mode,
    diff_text: Option<&str>,
    log_text: Option<&str>,
) -> String {
    match mode {
        Mode::DiffReview => {
            let Some(diff) = diff_text else {
                return user_input.to_string();
            };
            let diff = diff.trim();
            if diff.is_empty() {
                return user_input.to_string();
            }
            format!(
                "{user_input}\n\n以下が差分です。重要箇所を優先してレビューしてください。\n```diff\n{diff}\n```"
            )
        }
        Mode::LogAnalysis => {
            let Some(logs) = log_text else {
                return user_input.to_string();
            };
            let logs = logs.trim();
            if logs.is_empty() {
                return user_input.to_string();
            }
            format!(
                "{user_input}\n\nAnalyze the following log output and provide:\n1) A brief summary\n2) Errors/anomalies\n3) Potential root causes\n4) Recommended remediation steps\n\nLog output:\n{logs}"
            )
        }
        _ => user_input.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_is_injected_only_for_diff_review() {
        let user = "Review";
        let diff = "diff --git a/a b/a\n+hi\n";
        assert_eq!(compose_user_text(user, &Mode::Kabeuchi, Some(diff), None), user);
        let out = compose_user_text(user, &Mode::DiffReview, Some(diff), None);
        assert!(out.contains("```diff"));
        assert!(out.contains("diff --git"));
    }

    #[test]
    fn logs_are_injected_only_for_log_analysis() {
        let user = "Analyze";
        let logs = "ERROR boom";
        assert_eq!(compose_user_text(user, &Mode::Kabeuchi, None, Some(logs)), user);
        let out = compose_user_text(user, &Mode::LogAnalysis, None, Some(logs));
        assert!(out.contains("Log output:"));
        assert!(out.contains("ERROR boom"));
    }
}
