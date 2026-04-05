use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub default_profile: String,
    pub aliases: BTreeMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut aliases = BTreeMap::new();
        aliases.insert("dx".to_string(), "Developer Experience".to_string());
        aliases.insert("ops".to_string(), "Operations Crew".to_string());
        aliases.insert("qa".to_string(), "Quality Assurance".to_string());
        Self {
            default_profile: "friends".to_string(),
            aliases,
        }
    }
}

pub fn project_config_path(root: &Path) -> PathBuf {
    root.join(".mini-rust-cli.json")
}

pub fn resolve_profile_alias(root: &Path, requested: Option<&str>, config: &AppConfig) -> String {
    let _config_path = project_config_path(root);
    let key = requested
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(config.default_profile.as_str());

    config
        .aliases
        .get(key)
        .cloned()
        .unwrap_or_else(|| key.to_string())
}
