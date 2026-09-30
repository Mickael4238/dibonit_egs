//! Settings for EGS
//!
//! Provides configuration structures for EGS components.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// EGS configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Application name
    pub app_name: String,
    /// Application version
    pub app_version: String,
    /// Environment (development, staging, production)
    pub environment: Environment,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Network configuration
    pub network: NetworkConfig,
    /// ESB configuration
    pub esb: EsbConfig,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            app_name: "egs_app".to_string(),
            app_version: "0.0.0".to_string(),
            environment: Environment::Development,
            logging: LoggingConfig::default(),
            network: NetworkConfig::default(),
            esb: EsbConfig::default(),
        }
    }
}

/// Environment types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    /// Development environment
    Development,
    /// Staging environment
    Staging,
    /// Production environment
    Production,
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: LogLevel,
    /// Log file path (optional)
    pub file_path: Option<PathBuf>,
    /// Whether to log to console
    pub console: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            file_path: None,
            console: true,
        }
    }
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// Trace level
    Trace,
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warning level
    Warn,
    /// Error level
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Host address
    pub host: String,
    /// Port number
    pub port: u16,
    /// Whether to use SSL/TLS
    pub ssl: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            ssl: false,
        }
    }
}

/// ESB configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsbConfig {
    /// ESB service host
    pub host: String,
    /// ESB service port
    pub port: u16,
    /// Application trigram for identification
    pub app_trigram: String,
    /// Authentication token (optional)
    pub auth_token: Option<String>,
    /// Whether to use encryption
    pub encryption: bool,
}

impl Default for EsbConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8081,
            app_trigram: "APP".to_string(),
            auth_token: None,
            encryption: true,
        }
    }
}

impl Settings {
    /// Loads settings from a file
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let settings: Self = toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(settings)
    }

    /// Saves settings to a file
    pub fn to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), std::io::Error> {
        let content = toml::to_string(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
