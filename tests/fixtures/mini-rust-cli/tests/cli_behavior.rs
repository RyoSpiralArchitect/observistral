use mini_rust_cli::config::{AppConfig, project_config_path, resolve_profile_alias};
use mini_rust_cli::render::{RenderOptions, render_greeting, slugify_profile_label};
use std::path::Path;

#[test]
fn project_config_is_local_to_repo_root() {
    let path = project_config_path(Path::new("/tmp/mini-rust-cli"));
    assert!(path.ends_with(".mini-rust-cli.json"));
}

#[test]
fn aliases_are_loaded_from_project_config_context() {
    let config = AppConfig::default();
    let profile = resolve_profile_alias(Path::new("."), Some("dx"), &config);
    assert_eq!(profile, "Developer Experience");
}

#[test]
fn shout_turns_output_uppercase() {
    let output = render_greeting(
        "Developer Experience",
        RenderOptions {
            shout: true,
            include_slug: false,
        },
    );
    assert_eq!(output, "HELLO, DEVELOPER EXPERIENCE!");
}

#[test]
fn slug_collapses_repeated_separators() {
    assert_eq!(slugify_profile_label("Team   Alpha!!"), "team-alpha");
}
