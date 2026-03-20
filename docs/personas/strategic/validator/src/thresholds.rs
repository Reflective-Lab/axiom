//! Red-Flag Threshold Configuration
//!
//! Configurable thresholds for detecting pressure signatures and red-flag moments.
//! Thresholds are loaded from TOML configuration files with fallback to defaults.

use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Top-level threshold configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ThresholdConfig {
    #[serde(default)]
    pub version: String,

    #[serde(default)]
    pub active_profile: String,

    #[serde(default)]
    pub profiles: Profiles,

    #[serde(default)]
    pub red_flags: RedFlagConfig,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            active_profile: "balanced".to_string(),
            profiles: Profiles::default(),
            red_flags: RedFlagConfig::default(),
        }
    }
}

/// Three preset threshold profiles
#[derive(Debug, Clone, Deserialize)]
pub struct Profiles {
    #[serde(default)]
    pub conservative: ProfileThresholds,

    #[serde(default)]
    pub balanced: ProfileThresholds,

    #[serde(default)]
    pub aggressive: ProfileThresholds,
}

impl Default for Profiles {
    fn default() -> Self {
        Self {
            conservative: ProfileThresholds::conservative(),
            balanced: ProfileThresholds::balanced(),
            aggressive: ProfileThresholds::aggressive(),
        }
    }
}

/// Per-profile threshold settings
#[derive(Debug, Clone, Deserialize)]
pub struct ProfileThresholds {
    #[serde(default = "default_warn_spike_balanced")]
    pub warn_spike_threshold: usize,

    #[serde(default = "default_repeated_trigger")]
    pub repeated_trigger_threshold: usize,

    #[serde(default = "default_repeated_ack")]
    pub repeated_ack_threshold: usize,

    #[serde(default = "default_repeated_counter_voice")]
    pub repeated_counter_voice_threshold: usize,

    #[serde(default = "default_override_always_flags")]
    pub override_always_flags: bool,
}

fn default_warn_spike_balanced() -> usize { 3 }
fn default_repeated_trigger() -> usize { 2 }
fn default_repeated_ack() -> usize { 2 }
fn default_repeated_counter_voice() -> usize { 2 }
fn default_override_always_flags() -> bool { true }

impl Default for ProfileThresholds {
    fn default() -> Self {
        Self::balanced()
    }
}

impl ProfileThresholds {
    pub fn conservative() -> Self {
        Self {
            warn_spike_threshold: 5,
            repeated_trigger_threshold: 4,
            repeated_ack_threshold: 4,
            repeated_counter_voice_threshold: 4,
            override_always_flags: true,
        }
    }

    pub fn balanced() -> Self {
        Self {
            warn_spike_threshold: 3,
            repeated_trigger_threshold: 2,
            repeated_ack_threshold: 2,
            repeated_counter_voice_threshold: 2,
            override_always_flags: true,
        }
    }

    pub fn aggressive() -> Self {
        Self {
            warn_spike_threshold: 2,
            repeated_trigger_threshold: 1,
            repeated_ack_threshold: 1,
            repeated_counter_voice_threshold: 1,
            override_always_flags: true,
        }
    }
}

/// Red-flag detection configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RedFlagConfig {
    #[serde(default = "default_institutional_overrun_multiplier")]
    pub institutional_overrun_multiplier: f64,

    #[serde(default = "default_sell_scale_gates")]
    pub sell_scale_gates: Vec<String>,
}

fn default_institutional_overrun_multiplier() -> f64 { 2.0 }

fn default_sell_scale_gates() -> Vec<String> {
    vec![
        "customer-commitment".to_string(),
        "funding-narrative".to_string(),
    ]
}

impl Default for RedFlagConfig {
    fn default() -> Self {
        Self {
            institutional_overrun_multiplier: default_institutional_overrun_multiplier(),
            sell_scale_gates: default_sell_scale_gates(),
        }
    }
}

/// Load threshold configuration from TOML file
///
/// Returns default configuration if file doesn't exist or fails to parse.
///
/// # Arguments
///
/// * `config_path` - Optional path to TOML configuration file
pub fn load_thresholds(config_path: Option<&Path>) -> ThresholdConfig {
    match config_path {
        Some(path) if path.exists() => {
            match fs::read_to_string(path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!("Warning: Failed to parse threshold config ({e}), using defaults");
                        ThresholdConfig::default()
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read threshold config ({e}), using defaults");
                    ThresholdConfig::default()
                }
            }
        }
        _ => ThresholdConfig::default()
    }
}

/// Get active profile from configuration
///
/// Falls back to balanced profile if specified profile doesn't exist.
///
/// # Arguments
///
/// * `config` - Threshold configuration
pub fn get_active_profile(config: &ThresholdConfig) -> &ProfileThresholds {
    match config.active_profile.as_str() {
        "conservative" => &config.profiles.conservative,
        "balanced" => &config.profiles.balanced,
        "aggressive" => &config.profiles.aggressive,
        _ => {
            let profile = &config.active_profile;
            eprintln!("Warning: Unknown profile '{profile}', using balanced");
            &config.profiles.balanced
        }
    }
}
