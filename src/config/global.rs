use crate::asciistackstr::AsciiStackString;
use directories::ProjectDirs;
use serde::Deserialize;
use std::net::SocketAddr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Couldn't determine home directory")]
    UnknownHomeDirectory,
    #[error("Failed to read {0}: {1}")]
    Read(std::path::PathBuf, #[source] std::io::Error),
    #[error("Failed to parse {0}: {1}")]
    Parse(std::path::PathBuf, #[source] toml::de::Error),
}

pub fn load_config() -> Result<GlobalConfig, ConfigError> {
    let dirs = ProjectDirs::from("net.octyl", "Octavia Togami", "audio-bicycle")
        .ok_or(ConfigError::UnknownHomeDirectory)?;

    let config_file = dirs.config_dir().join("config.toml");
    let config_text = std::fs::read_to_string(&config_file)
        .map_err(|e| ConfigError::Read(config_file.clone(), e))?;
    toml::from_str(&config_text).map_err(|e| ConfigError::Parse(config_file, e))
}

#[derive(Debug, Clone, Deserialize)]
pub struct GlobalConfig {
    pub local_address: SocketAddr,
    pub dest_address: SocketAddr,
    pub stream_name: AsciiStackString<16>,
}
