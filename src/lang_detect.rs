// Lightweight language heuristics used for server-side enforcement.
// This deliberately avoids heavy dependencies and keeps false positives low.
//
// NOTE: This is a heuristic, not a classifier. It is used only to trigger a one-shot
// "rewrite into requested language" retry for Observer responses.

fn count_japanese_chars(s: &str) -> usize {
    s.chars()
        .filter(|&ch| {
            matches!(
                ch as u32,
                0x3040..=0x309F | // Hiragana
                0x30A0..=0x30FF | // Katakana
                0x3400..=0x4DBF | // CJK Ext A
                0x4E00..=0x9FFF // CJK Unified
            )
        })
        .count()
}

fn count_latin_letters(s: &str) -> usize {
    s.chars().filter(|c| c.is_ascii_alphabetic()).count()
}

fn is_latin_accent(ch: char) -> bool {
    // Keep this broad and dependency-free. Latin-1 Supplement + Latin Extended-A covers
    // common French diacritics and ligatures.
    matches!(
        ch as u32,
        0x00C0..=0x00FF | // Latin-1 Supplement
        0x0100..=0x017F // Latin Extended-A
    )
}

fn count_french_accents(s: &str) -> usize {
    s.chars().filter(|&c| is_latin_accent(c)).count()
}

fn tokenize_words_lower(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphabetic())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect()
}

fn strip_fenced_code_blocks(s: &str) -> String {
    // Remove fenced code blocks (```...```) because they are often English-heavy and can
    // overwhelm lightweight language heuristics.
    //
    // This is only used for Observer language enforcement (rewrite retry trigger).
    let mut out = String::with_capacity(s.len());
    let mut in_fence = false;

    for line in s.lines() {
        let t = line.trim_start();
        if t.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }

    out
}

fn strip_structured_observer_blocks(s: &str) -> String {
    // Observer responses append machine-readable blocks that intentionally contain
    // English-heavy keys (e.g. title/to_coder/severity/score). Those keys should not
    // cause a Japanese/French response to be rejected.
    //
    // Keep only the "human" critique portion before any structured block header.
    let mut out = String::with_capacity(s.len());
    for line in s.lines() {
        let t = line.trim();
        if t.starts_with("---") {
            let low = t.trim_matches('-').trim().to_ascii_lowercase();
            if low.starts_with("phase")
                || low.starts_with("proposals")
                || low.starts_with("critical_path")
                || low.starts_with("health")
            {
                break;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

pub fn is_skippable_for_lang_check(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return true;
    }
    if t.starts_with("[Observer]") {
        return true;
    }
    let lower = t.to_lowercase();
    if lower.starts_with("[error]") || lower.starts_with("[erreur]") {
        return true;
    }
    // Japanese error prefix used by the UI.
    if t.starts_with("[エラー]") {
        return true;
    }
    false
}

pub fn looks_japanese(s: &str) -> bool {
    let stripped = strip_structured_observer_blocks(s);
    let stripped = strip_fenced_code_blocks(&stripped);
    let jp = count_japanese_chars(&stripped);
    let lat = count_latin_letters(&stripped);
    if jp < 8 {
        return false;
    }
    if lat == 0 {
        return true;
    }
    // Allow some English tokens (code, keys) but avoid "mostly English with a few JP chars".
    lat <= jp * 2
}

pub fn looks_french(s: &str) -> bool {
    let stripped = strip_structured_observer_blocks(s);
    let stripped = strip_fenced_code_blocks(&stripped);
    let accents = count_french_accents(&stripped);
    let toks = tokenize_words_lower(&stripped);
    if toks.is_empty() {
        return false;
    }

    const FR: [&str; 21] = [
        "le", "la", "les", "des", "du", "de", "pour", "avec", "sans", "est", "sont", "pas", "mais",
        "donc", "sur", "dans", "vous", "tu", "je", "nous", "votre",
    ];
    const EN: [&str; 16] = [
        "the", "and", "you", "your", "should", "this", "that", "with", "for", "not", "are", "is",
        "was", "were", "will", "can",
    ];

    let mut fr = 0usize;
    let mut en = 0usize;
    for t in toks {
        if FR.contains(&t.as_str()) {
            fr += 1;
        }
        if EN.contains(&t.as_str()) {
            en += 1;
        }
    }

    if accents > 0 && fr >= 1 {
        return true;
    }
    fr > en + 1
}

pub fn needs_language_rewrite(expected: &str, content: &str) -> bool {
    if is_skippable_for_lang_check(content) {
        return false;
    }
    let e = expected.trim().to_ascii_lowercase();
    if e == "en" {
        return false;
    }
    if e == "fr" {
        return !looks_french(content);
    }
    // Default: Japanese.
    !looks_japanese(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn japanese_heuristic_basic() {
        assert!(looks_japanese("これはテストです。バグを直してください。"));
        assert!(!looks_japanese(
            "Coder attempted to create a repo but failed."
        ));
    }

    #[test]
    fn fenced_code_does_not_break_japanese_detection() {
        let s = "これはテストです。\n```rust\nfn main() { println!(\"hello world\"); }\n```\nバグを直してください。";
        assert!(looks_japanese(s));
    }

    #[test]
    fn structured_blocks_do_not_break_japanese_detection() {
        let s = "これは日本語の批評です。改善点を指摘します。\n\n--- phase ---\ncore\n\n--- proposals ---\n1) title: Fix error handling\n   to_coder: Add retries.\n   severity: warn\n   score: 70\n   phase: core\n   impact: stability\n   cost: low\n";
        assert!(looks_japanese(s));
    }

    #[test]
    fn french_heuristic_basic() {
        assert!(looks_french(
            "Ceci est un test. Vous devez corriger ce bug."
        ));
        assert!(!looks_french("This is a test and you should fix it."));
    }

    #[test]
    fn fenced_code_does_not_break_french_detection() {
        let s =
            "Ceci est un test.\n```python\nprint('hello world')\n```\nVous devez corriger ce bug.";
        assert!(looks_french(s));
    }

    #[test]
    fn structured_blocks_do_not_break_french_detection() {
        let s = "Ceci est une critique en français.\n\n--- phase ---\ncore\n\n--- proposals ---\n1) title: Fix retries\n   to_coder: Implement exponential backoff.\n   severity: warn\n   score: 80\n   phase: core\n   impact: reliability\n   cost: medium\n";
        assert!(looks_french(s));
    }

    #[test]
    fn skippable_blocks_do_not_trigger_rewrite() {
        assert!(!needs_language_rewrite(
            "ja",
            "[Observer] No new critique. Loop detected."
        ));
        assert!(!needs_language_rewrite("fr", "[error] HTTP 401"));
        assert!(!needs_language_rewrite("ja", "[エラー] HTTP 401"));
    }
}
