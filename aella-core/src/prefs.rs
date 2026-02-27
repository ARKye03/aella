use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPrefs {
    pub sidebar_collapsed: bool,
    pub errors_panel_collapsed: bool,
}

impl Default for UiPrefs {
    fn default() -> Self {
        Self {
            sidebar_collapsed: false,
            errors_panel_collapsed: true,
        }
    }
}

impl UiPrefs {
    pub fn load() -> Self {
        let path = Self::path();
        let Ok(contents) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        serde_json::from_str(&contents).unwrap_or_default()
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string(self) {
            Ok(json) => {
                if let Err(err) = std::fs::write(&path, json) {
                    eprintln!("[aella] failed to save ui prefs: {err}");
                }
            }
            Err(err) => eprintln!("[aella] failed to serialize ui prefs: {err}"),
        }
    }

    fn path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("aella").join("ui_prefs.json")
    }
}
