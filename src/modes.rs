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
        Mode::Observer => "You are an observer. Watch the Coder's progress, narrate what is happening, and critique risks calmly.\n\nConstraints:\n- Do NOT issue direct orders to the Coder.\n- Output both blocks below whenever you have something meaningful to say.\n\n--- phase ---\ncore\n(replace with one word: core=foundational logic not yet stable, feature=adding capabilities, polish=refining UX/tests/docs/perf)\n\n--- proposals ---\n1) title: <short title>\n   to_coder: <message to send>\n   severity: info|warn|crit\n   score: <0-100 overall priority: 100=critical blocker, 50=useful, 10=nice-to-have>\n   phase: core|feature|polish|any\n   impact: <one line: what this fixes or improves>\n   cost: low|medium|high\n\nKeep the main body conversational. Sort proposals by score descending.",
        Mode::DiffReview => "あなたはコードレビュアーです。入力されたdiffを読み、リスク・改善案・テスト観点を批評してください。",
        Mode::Vibe => "あなたは熟練のソフトウェアエンジニアです。ユーザーがVIBEコーディングできるように、最短で動く実装案を提示してください。",
        Mode::LogAnalysis => "You are an expert site-reliability engineer and observability specialist. When given log output or metrics, identify errors, anomalies, performance bottlenecks, and actionable remediation steps. Be concise and structured.",
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
