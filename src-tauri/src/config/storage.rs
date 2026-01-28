use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppConfig {
    pub routes: Vec<crate::audio::Route>,
    pub device_gains: HashMap<String, f32>,
}

pub struct ConfigStorage {
    config_dir: PathBuf,
}

impl ConfigStorage {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = directories::ProjectDirs::from("com", "audioflow", "Audio Flow")
            .expect("无法获取配置目录")
            .config_dir()
            .to_path_buf();
        
        fs::create_dir_all(&config_dir)?;
        
        Ok(Self { config_dir })
    }
    
    pub fn save_config(&self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = self.config_dir.join("config.toml");
        let toml_string = toml::to_string_pretty(config)?;
        fs::write(config_path, toml_string)?;
        Ok(())
    }
    
    pub fn load_config(&self) -> Result<Option<AppConfig>, Box<dyn std::error::Error>> {
        let config_path = self.config_dir.join("config.toml");
        if !config_path.exists() {
            return Ok(None);
        }
        
        let contents = fs::read_to_string(config_path)?;
        let config: AppConfig = toml::from_str(&contents)?;
        Ok(Some(config))
    }
}
