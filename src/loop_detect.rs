use std::collections::HashSet;

fn is_cjk(c: char) -> bool {
    let u = c as u32;
    // Hiragana / Katakana
    if (0x3040..=0x30FF).contains(&u) {
        return true;
    }
    // CJK Unified Ideographs (incl. Extension A)
    (0x3400..=0x9FFF).contains(&u)
}

fn is_latin_ext(c: char) -> bool {
    let u = c as u32;
    (0x00C0..=0x024F).contains(&u)
}

fn strip_fences_and_inline_code(s: &str) -> String {
    // Remove ```fenced``` blocks and `inline code` to reduce false positives.
    // This is intentionally simple and streaming-safe (no regex dependency).
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(chars.len());

    let mut i = 0usize;
    let mut in_fence = false;
    let mut in_inline = false;

    while i < chars.len() {
        // Toggle fenced blocks on ```
        if i + 2 < chars.len() && chars[i] == '`' && chars[i + 1] == '`' && chars[i + 2] == '`' {
            in_fence = !in_fence;
            i += 3;
            continue;
        }
        if in_fence {
            i += 1;
            continue;
        }

        // Toggle inline code on `
        if chars[i] == '`' {
            in_inline = !in_inline;
            i += 1;
            continue;
        }
        if in_inline {
            i += 1;
            continue;
        }

        out.push(chars[i]);
        i += 1;
    }
    out
}

fn strip_urls(s: &str) -> String {
    // Remove http(s)://... until whitespace.
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(chars.len());
    let mut i = 0usize;
    while i < chars.len() {
        let rem = &chars[i..];
        let starts_http = rem.len() >= 7
            && rem[0].to_ascii_lowercase() == 'h'
            && rem[1].to_ascii_lowercase() == 't'
            && rem[2].to_ascii_lowercase() == 't'
            && rem[3].to_ascii_lowercase() == 'p'
            && (rem[4] == ':' && rem[5] == '/' && rem[6] == '/');
        let starts_https = rem.len() >= 8
            && rem[0].to_ascii_lowercase() == 'h'
            && rem[1].to_ascii_lowercase() == 't'
            && rem[2].to_ascii_lowercase() == 't'
            && rem[3].to_ascii_lowercase() == 'p'
            && rem[4].to_ascii_lowercase() == 's'
            && (rem[5] == ':' && rem[6] == '/' && rem[7] == '/');
        if starts_http || starts_https {
            // Skip until whitespace.
            while i < chars.len() && !chars[i].is_whitespace() {
                i += 1;
            }
            out.push(' ');
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

pub fn normalize_for_sim(s: &str) -> String {
    let mut t = strip_fences_and_inline_code(s);
    t = strip_urls(&t);

    let mut out = String::with_capacity(t.len());
    let mut prev_space = true;
    for mut c in t.chars() {
        if c.is_ascii_uppercase() {
            c = c.to_ascii_lowercase();
        }
        if c.is_whitespace() {
            c = ' ';
        }

        let allowed = c == ' ' || c.is_ascii_alphanumeric() || is_latin_ext(c) || is_cjk(c);
        if !allowed {
            c = ' ';
        }

        if c == ' ' {
            if prev_space {
                continue;
            }
            prev_space = true;
            out.push(' ');
        } else {
            prev_space = false;
            out.push(c);
        }
    }
    out.trim().to_string()
}

pub fn token_set_for_sim(s: &str) -> HashSet<String> {
    let t = normalize_for_sim(s);
    let mut out: HashSet<String> = HashSet::new();
    if t.is_empty() {
        return out;
    }

    for w in t.split(' ') {
        let w = w.trim();
        if w.len() >= 2 {
            out.insert(w.to_string());
        }
    }

    // For CJK-heavy text, add bigrams to detect repetition without spaces.
    let cjk_chars: Vec<char> = t.chars().filter(|&c| is_cjk(c)).collect();
    if cjk_chars.len() >= 8 {
        for i in 0..cjk_chars.len().saturating_sub(1) {
            if out.len() >= 2400 {
                break;
            }
            out.insert(format!("{}{}", cjk_chars[i], cjk_chars[i + 1]));
        }
    }

    out
}

pub fn jaccard_sim(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let mut inter: usize = 0;
    for x in a.iter() {
        if b.contains(x) {
            inter += 1;
        }
    }
    let union = a.len() + b.len() - inter;
    if union == 0 {
        0.0
    } else {
        inter as f64 / union as f64
    }
}

pub fn similarity(a: &str, b: &str) -> f64 {
    jaccard_sim(&token_set_for_sim(a), &token_set_for_sim(b))
}

pub fn is_skippable_for_loop(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return true;
    }
    if t.starts_with("[Observer]") {
        return true;
    }
    let low = t.to_ascii_lowercase();
    if low.starts_with("[error]") || low.starts_with("[erreur]") || t.starts_with("[エラー]") {
        return true;
    }
    false
}
