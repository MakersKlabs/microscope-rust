//! Application settings with JSON persistence.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Per-camera settings (remembered by camera name).
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CameraSettings {
    #[serde(default)]
    pub rotation: u16,
    #[serde(default)]
    pub flip_h: bool,
    #[serde(default)]
    pub flip_v: bool,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub fps: Option<u32>,
}

fn default_language() -> String {
    "system".to_string()
}

fn default_bitrate() -> u32 {
    8000
}

fn default_true() -> bool {
    true
}

/// Global settings persisted in the user configuration directory.
#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    #[serde(default)]
    pub cameras: HashMap<String, CameraSettings>,
    #[serde(default)]
    pub selected_camera: Option<String>,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub last_image_dir: Option<String>,
    #[serde(default)]
    pub last_video_dir: Option<String>,
    #[serde(default = "default_bitrate")]
    pub bitrate_kbps: u32,
    #[serde(default = "default_true")]
    pub crt_effects: bool,
    #[serde(default)]
    pub window_size: Option<[f32; 2]>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            cameras: HashMap::new(),
            selected_camera: None,
            language: default_language(),
            last_image_dir: None,
            last_video_dir: None,
            bitrate_kbps: default_bitrate(),
            crt_effects: true,
            window_size: None,
        }
    }
}

impl Settings {
    /// Path of the JSON settings file for the current platform.
    pub fn config_path() -> PathBuf {
        directories::ProjectDirs::from("org", "MakersKlabs", "microscope-rust")
            .map(|dirs| dirs.config_dir().join("settings.json"))
            .unwrap_or_else(|| PathBuf::from("microscope-rust-settings.json"))
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let tmp = path.with_extension("json.tmp");
            if std::fs::write(&tmp, json).is_ok() {
                let _ = std::fs::rename(&tmp, &path);
            }
        }
    }

    pub fn camera(&self, key: &str) -> CameraSettings {
        self.cameras.get(key).cloned().unwrap_or_default()
    }

    pub fn set_camera(&mut self, key: &str, value: CameraSettings) {
        self.cameras.insert(key.to_string(), value);
    }

    /// Last used image directory (existing) or the platform pictures folder.
    pub fn image_dir(&self) -> PathBuf {
        self.last_image_dir
            .as_ref()
            .map(PathBuf::from)
            .filter(|path| path.is_dir())
            .unwrap_or_else(crate::util::image_dir)
    }

    /// Last used video directory (existing) or the platform videos folder.
    pub fn video_dir(&self) -> PathBuf {
        self.last_video_dir
            .as_ref()
            .map(PathBuf::from)
            .filter(|path| path.is_dir())
            .unwrap_or_else(crate::util::video_dir)
    }
}
