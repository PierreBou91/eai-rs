use once_cell::sync::Lazy;
use std::{env, path::PathBuf};
use tracing::{error, Level};

#[derive(Clone, Debug)]
pub struct UserSettings {
    /// verbose mode
    pub log_level: Level,
    /// the calling Application Entity title
    pub calling_ae_title: String,
    /// enforce max pdu length
    pub strict: bool,
    /// Only accept native/uncompressed transfer syntaxes
    pub uncompressed_only: bool,
    /// max pdu length
    pub max_pdu_length: u32,
    /// output directory for incoming objects
    pub out_dir: PathBuf,
    /// Which port to listen on
    pub port: u16,
}

/// default settings if they are not provided by the user
pub static DEFAULT_SETTINGS: Lazy<UserSettings> = Lazy::new(|| UserSettings {
    // Lazy is required for "PACS".to_string() and PathBuf::from(".") to work
    log_level: Level::WARN,
    calling_ae_title: "PACS".to_string(),
    strict: false,
    uncompressed_only: false,
    max_pdu_length: 16352,
    out_dir: PathBuf::from("."),
    port: 11112,
});

impl UserSettings {
    /// a new UserSettings object with default values
    pub fn new() -> UserSettings {
        DEFAULT_SETTINGS.clone()
    }

    /// a new UserSettings object with values from the environment variables
    pub fn settings_from_envars() -> Result<UserSettings, Box<dyn std::error::Error>> {
        let log_level = env::var("LOG_LEVEL")
            .unwrap_or_else(|_| DEFAULT_SETTINGS.log_level.to_string())
            .parse::<Level>()
            .unwrap_or_else(|e| {
                error!("Failed to parse the LOG_LEVEL environment variable: {}", e);
                DEFAULT_SETTINGS.log_level
            });

        let calling_ae_title =
            env::var("AE_TITLE").unwrap_or_else(|_| DEFAULT_SETTINGS.calling_ae_title.clone());

        let strict = env::var("ENFORCE_MAX_PDU_LENGTH")
            .unwrap_or_else(|_| DEFAULT_SETTINGS.strict.to_string())
            .parse::<bool>()
            .unwrap_or_else(|e| {
                error!("Failed to parse the STRICT environment variable: {}", e);
                DEFAULT_SETTINGS.strict
            });

        let uncompressed_only = env::var("UNCOMPRESSED_ONLY")
            .unwrap_or_else(|_| DEFAULT_SETTINGS.uncompressed_only.to_string())
            .parse::<bool>()
            .unwrap_or_else(|e| {
                error!(
                    "Failed to parse the UNCOMPRESSED_ONLY environment variable: {}",
                    e
                );
                DEFAULT_SETTINGS.uncompressed_only
            });

        let max_pdu_length = env::var("MAX_PDU_LENGTH")
            .unwrap_or_else(|_| DEFAULT_SETTINGS.max_pdu_length.to_string())
            .parse::<u32>()
            .unwrap_or_else(|e| {
                error!(
                    "Failed to parse the MAX_PDU_LENGTH environment variable: {}",
                    e
                );
                DEFAULT_SETTINGS.max_pdu_length
            });

        let out_dir = env::var("OUT_DIR")
            .unwrap_or_else(|_| DEFAULT_SETTINGS.out_dir.to_str().unwrap().to_string())
            .parse::<String>()
            .unwrap_or_else(|e| {
                error!("Failed to parse the OUT_DIR environment variable: {}", e);
                DEFAULT_SETTINGS.out_dir.to_str().unwrap().to_string()
            })
            .into();

        let port = env::var("PORT")
            .unwrap_or_else(|_| DEFAULT_SETTINGS.port.to_string())
            .parse::<u16>()
            .unwrap_or_else(|e| {
                error!("Failed to parse the PACS_PORT environment variable: {}", e);
                DEFAULT_SETTINGS.port
            });

        Ok(UserSettings {
            log_level,
            calling_ae_title,
            strict,
            uncompressed_only,
            max_pdu_length,
            out_dir,
            port,
        })
    }
}

pub fn load_settings() -> Result<UserSettings, Box<dyn std::error::Error>> {
    UserSettings::settings_from_envars()
}
