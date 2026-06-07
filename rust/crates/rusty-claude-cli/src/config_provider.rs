use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use serde_json::Value as JsonValue;

/// Load all config files and apply provider configurations to environment
pub fn apply_provider_config_from_files() {
    let mut merged = BTreeMap::new();
    
    // Load config files in order of precedence (lowest to highest)
    // 0. System level global config /usr/local/etc/claw/settings.json

    let system_config = PathBuf::from("/usr/local/etc/claw/settings.json");

    merge_config_file(&system_config, &mut merged);

    // 1. User global config ~/.config/claw/settings.json
    if let Ok(home_dir) = std::env::var("HOME") {
        let home_path = PathBuf::from(home_dir);
        let global_config = home_path.join(".config").join("claw").join("settings.json");
        merge_config_file(&global_config, &mut merged);
        
        // ~/.claw/settings.json
        let user_claw_settings = home_path.join(".claw").join("settings.json");
        merge_config_file(&user_claw_settings, &mut merged);
        
        // Legacy ~/.claw.json
        let legacy_global = home_path.join(".claw.json");
        merge_config_file(&legacy_global, &mut merged);
    }
    
    // 2. Project level config .claw.json and .claw/settings.json
    if let Ok(cwd) = std::env::current_dir() {
        let project_claw_json = cwd.join(".claw.json");
        merge_config_file(&project_claw_json, &mut merged);
        
        let project_settings = cwd.join(".claw").join("settings.json");
        merge_config_file(&project_settings, &mut merged);
        
        // 3. Local override .claw/settings.local.json
        let local_settings = cwd.join(".claw").join("settings.local.json");
        merge_config_file(&local_settings, &mut merged);
    }
    
    // Apply provider config to environment
    apply_provider_config_to_env(&merged);
}

/// Merge a single config file into the merged map
fn merge_config_file(path: &Path, merged: &mut BTreeMap<String, JsonValue>) {
    if path.exists() && path.is_file() {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(JsonValue::Object(obj)) = serde_json::from_str(&content) {
                for (k, v) in obj {
                    merged.insert(k, v);
                }
            }
        }
    }
}

/// Apply provider configurations from merged config to process environment
/// Environment variables take precedence over config file values
fn apply_provider_config_to_env(merged: &BTreeMap<String, JsonValue>) {
    // Apply env section first
    if let Some(JsonValue::Object(env)) = merged.get("env") {
        for (key, value) in env {
            if let JsonValue::String(val) = value {
                if std::env::var(key).is_err() {
                    std::env::set_var(key, val);
                }
            }
        }
    }

    if let Some(JsonValue::Object(providers)) = merged.get("providers") {
        for (provider_name, config) in providers {
            let provider_upper = provider_name.to_uppercase();
            if let JsonValue::Object(config_obj) = config {
                // Apply api_key if not already set in environment
                if let Some(JsonValue::String(api_key)) = config_obj.get("api_key") {
                    let env_key = format!("{}_API_KEY", provider_upper);
                    if std::env::var(&env_key).is_err() {
                        std::env::set_var(&env_key, api_key);
                    }
                }
                // Apply base_url if not already set in environment
                if let Some(JsonValue::String(base_url)) = config_obj.get("base_url") {
                    let env_key = format!("{}_BASE_URL", provider_upper);
                    if std::env::var(&env_key).is_err() {
                        std::env::set_var(&env_key, base_url);
                    }
                }
                // Apply auth_token if not already set in environment (for Anthropic)
                if let Some(JsonValue::String(auth_token)) = config_obj.get("auth_token") {
                    let env_key = format!("{}_AUTH_TOKEN", provider_upper);
                    if std::env::var(&env_key).is_err() {
                        std::env::set_var(&env_key, auth_token);
                    }
                }
            }
        }
    }
}
