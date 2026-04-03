#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RenderOptions {
    pub shout: bool,
    pub include_slug: bool,
}

pub fn render_greeting(profile_display: &str, options: RenderOptions) -> String {
    let mut message = format!("Hello, {profile_display}!");
    if options.include_slug {
        let slug = slugify_profile_label(profile_display);
        message.push_str(&format!(" [{slug}]"));
    }
    if options.shout {
        message = message.to_uppercase();
    }
    message
}

pub fn slugify_profile_label(input: &str) -> String {
    let mut slug = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else {
            slug.push('-');
        }
    }
    slug.trim_matches('-').to_string()
}
