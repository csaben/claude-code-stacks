use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use dirs::home_dir;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StacksConfig {
    pub tmux_strategy: TmuxStrategy,
    pub prompt_for_strategy: bool,
    pub in_tmux_behavior: InTmuxBehavior,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum InTmuxBehavior {
    #[serde(rename = "new-windows")]
    NewWindows,
    #[serde(rename = "new-session")]
    NewSession,
    #[serde(rename = "ask")]
    Ask,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TmuxStrategy {
    #[serde(rename = "separate-sessions")]
    SeparateSessions,
    #[serde(rename = "quad-split")]
    QuadSplit,
    #[serde(rename = "horizontal-split")]
    HorizontalSplit,
    #[serde(rename = "multiple-windows")]
    MultipleWindows,
}

impl Default for StacksConfig {
    fn default() -> Self {
        Self {
            tmux_strategy: TmuxStrategy::SeparateSessions,
            prompt_for_strategy: false,
            in_tmux_behavior: InTmuxBehavior::NewWindows,
        }
    }
}

impl TmuxStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            TmuxStrategy::SeparateSessions => "separate-sessions",
            TmuxStrategy::QuadSplit => "quad-split",
            TmuxStrategy::HorizontalSplit => "horizontal-split",
            TmuxStrategy::MultipleWindows => "multiple-windows",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "separate-sessions" => Ok(TmuxStrategy::SeparateSessions),
            "quad-split" => Ok(TmuxStrategy::QuadSplit),
            "horizontal-split" => Ok(TmuxStrategy::HorizontalSplit),
            "multiple-windows" => Ok(TmuxStrategy::MultipleWindows),
            _ => anyhow::bail!("Invalid tmux strategy: {}", s),
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TmuxStrategy::SeparateSessions => "Separate tmux sessions (current default)",
            TmuxStrategy::QuadSplit => "2x2 grid in one session (up to 4 claudes)",
            TmuxStrategy::HorizontalSplit => "4 horizontal panes in one session", 
            TmuxStrategy::MultipleWindows => "Multiple windows in same session",
        }
    }
}

pub fn get_config_path() -> Result<PathBuf> {
    let home = home_dir().context("Could not find home directory")?;
    let config_dir = home.join(".config").join("stacks");
    std::fs::create_dir_all(&config_dir)
        .context("Failed to create config directory")?;
    Ok(config_dir.join("config.toml"))
}

pub fn load_config() -> Result<StacksConfig> {
    let config_path = get_config_path()?;
    
    if !config_path.exists() {
        let default_config = StacksConfig::default();
        save_config(&default_config)?;
        return Ok(default_config);
    }

    let content = std::fs::read_to_string(&config_path)
        .context("Failed to read config file")?;
    
    // Try parsing as current config format first
    if let Ok(config) = toml::from_str::<StacksConfig>(&content) {
        return Ok(config);
    }
    
    // If that fails, try parsing as legacy format and migrate
    if let Ok(legacy_config) = toml::from_str::<LegacyConfig>(&content) {
        println!("Migrating config to new format...");
        let migrated_config = StacksConfig {
            tmux_strategy: legacy_config.tmux_strategy,
            prompt_for_strategy: legacy_config.prompt_for_strategy,
            in_tmux_behavior: InTmuxBehavior::NewWindows, // Default for migration
        };
        save_config(&migrated_config)?;
        return Ok(migrated_config);
    }
    
    // If all parsing fails, create default config
    println!("Config file format not recognized, creating new default config...");
    let default_config = StacksConfig::default();
    save_config(&default_config)?;
    Ok(default_config)
}

#[derive(Deserialize)]
struct LegacyConfig {
    pub tmux_strategy: TmuxStrategy,
    pub prompt_for_strategy: bool,
}

pub fn save_config(config: &StacksConfig) -> Result<()> {
    let config_path = get_config_path()?;
    let content = toml::to_string_pretty(config)
        .context("Failed to serialize config")?;
    
    std::fs::write(&config_path, content)
        .context("Failed to write config file")?;
    
    Ok(())
}

pub fn update_config<F>(updater: F) -> Result<StacksConfig> 
where
    F: FnOnce(&mut StacksConfig),
{
    let mut config = load_config()?;
    updater(&mut config);
    save_config(&config)?;
    Ok(config)
}

impl InTmuxBehavior {
    pub fn as_str(&self) -> &'static str {
        match self {
            InTmuxBehavior::NewWindows => "new-windows",
            InTmuxBehavior::NewSession => "new-session", 
            InTmuxBehavior::Ask => "ask",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "new-windows" => Ok(InTmuxBehavior::NewWindows),
            "new-session" => Ok(InTmuxBehavior::NewSession),
            "ask" => Ok(InTmuxBehavior::Ask),
            _ => anyhow::bail!("Invalid in-tmux behavior: {}", s),
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            InTmuxBehavior::NewWindows => "Create new windows in current session",
            InTmuxBehavior::NewSession => "Always create new session",
            InTmuxBehavior::Ask => "Ask what to do when already in tmux",
        }
    }
}