use std::{env, net::SocketAddr, path::PathBuf};

use secrecy::SecretString;
use thiserror::Error;
use url::Url;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub http: HttpConfig,
    pub storage: StorageConfig,
    pub security: SecurityConfig,
    pub observability: ObservabilityConfig,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let http = HttpConfig::from_env()?;
        let storage = StorageConfig::from_env()?;
        let security = SecurityConfig::from_env()?;
        let observability = ObservabilityConfig::from_env();
        Ok(Self {
            http,
            storage,
            security,
            observability,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpConfig {
    pub bind: SocketAddr,
    pub public_base_url: Url,
}

impl HttpConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let bind = env_value("RNOVEMAIL_BIND", "127.0.0.1:18089").parse()?;
        let public_base_url = Url::parse(&required_env("RNOVEMAIL_PUBLIC_BASE_URL")?)?;
        Ok(Self {
            bind,
            public_base_url,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
}

impl StorageConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let data_dir = PathBuf::from(env_value("RNOVEMAIL_DATA_DIR", "./data"));
        Ok(Self { data_dir })
    }
}

#[derive(Clone, Debug)]
pub struct SecurityConfig {
    pub master_key_file: PathBuf,
    pub bootstrap_admin_token: Option<SecretString>,
}

impl SecurityConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let master_key_file = PathBuf::from(required_env("RNOVEMAIL_MASTER_KEY_FILE")?);
        let bootstrap_admin_token = optional_secret("RNOVEMAIL_BOOTSTRAP_ADMIN_TOKEN");
        Ok(Self {
            master_key_file,
            bootstrap_admin_token,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservabilityConfig {
    pub log_format: LogFormat,
}

impl ObservabilityConfig {
    fn from_env() -> Self {
        Self {
            log_format: LogFormat::from_env_value(&env_value("RNOVEMAIL_LOG_FORMAT", "compact")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogFormat {
    Compact,
    Json,
}

impl LogFormat {
    fn from_env_value(value: &str) -> Self {
        match value.eq_ignore_ascii_case("json") {
            true => Self::Json,
            false => Self::Compact,
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable {0}")]
    MissingEnv(&'static str),
    #[error("bind address is invalid")]
    InvalidBind(#[from] std::net::AddrParseError),
    #[error("public base url is invalid")]
    InvalidUrl(#[from] url::ParseError),
}

fn required_env(name: &'static str) -> Result<String, ConfigError> {
    env::var(name).map_err(|_| ConfigError::MissingEnv(name))
}

fn env_value(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

fn optional_secret(name: &str) -> Option<SecretString> {
    env::var(name).ok().map(SecretString::new)
}
