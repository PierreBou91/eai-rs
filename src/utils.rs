use once_cell::sync::Lazy;
use std::{env, path::PathBuf};

#[derive(Clone)]
pub struct UserSettings {
    /// verbose mode
    pub verbose: bool,
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
    verbose: false,
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

    pub fn settings_from_envars() -> Result<UserSettings, Box<dyn std::error::Error>> {
        let default_calling_ae_title = "PACS".to_string();

        let calling_ae_title =
            env::var("PACS_AE_TITLE").unwrap_or_else(|_| default_calling_ae_title);

        let pacs_port = env::var("PACS_PORT")
            .unwrap_or_else(|_| DEFAULT_SETTINGS.port.to_string())
            .parse::<u16>()
            .unwrap_or_else(|e| {
                eprintln!("Failed to parse the PACS_PORT environment variable: {}", e);
                DEFAULT_SETTINGS.port
            });

        Ok(UserSettings {
            verbose: true,
            calling_ae_title: calling_ae_title,
            strict: true,
            uncompressed_only: false,
            max_pdu_length: 16384,
            out_dir: PathBuf::from("out"),
            port: pacs_port,
        })
    }
}

/// A list of supported abstract syntaxes for storage services
pub static ABSTRACT_SYNTAXES: &[&str] = &[
    "1.2.840.10008.5.1.4.1.1.2",
    "1.2.840.10008.5.1.4.1.1.2.1",
    "1.2.840.10008.5.1.4.1.1.9",
    "1.2.840.10008.5.1.4.1.1.8",
    "1.2.840.10008.5.1.4.1.1.7",
    "1.2.840.10008.5.1.4.1.1.6",
    "1.2.840.10008.5.1.4.1.1.5",
    "1.2.840.10008.5.1.4.1.1.4",
    "1.2.840.10008.5.1.4.1.1.4.1",
    "1.2.840.10008.5.1.4.1.1.4.2",
    "1.2.840.10008.5.1.4.1.1.4.3",
    "1.2.840.10008.5.1.4.1.1.3",
    "1.2.840.10008.5.1.4.1.1.2",
    "1.2.840.10008.5.1.4.1.1.1",
    "1.2.840.10008.5.1.4.1.1.1.1",
    "1.2.840.10008.5.1.4.1.1.1.1.1",
    "1.2.840.10008.5.1.4.1.1.104.1",
    "1.2.840.10008.5.1.4.1.1.104.2",
    "1.2.840.10008.5.1.4.1.1.104.3",
    "1.2.840.10008.5.1.4.1.1.11.1",
    "1.2.840.10008.5.1.4.1.1.128",
    "1.2.840.10008.5.1.4.1.1.13.1.3",
    "1.2.840.10008.5.1.4.1.1.13.1.4",
    "1.2.840.10008.5.1.4.1.1.13.1.5",
    "1.2.840.10008.5.1.4.1.1.130",
    "1.2.840.10008.5.1.4.1.1.481.1",
    "1.2.840.10008.5.1.4.1.1.20",
    "1.2.840.10008.5.1.4.1.1.3.1",
    "1.2.840.10008.5.1.4.1.1.7",
    "1.2.840.10008.5.1.4.1.1.7.1",
    "1.2.840.10008.5.1.4.1.1.7.2",
    "1.2.840.10008.5.1.4.1.1.7.3",
    "1.2.840.10008.5.1.4.1.1.7.4",
    "1.2.840.10008.5.1.4.1.1.88.11",
    "1.2.840.10008.5.1.4.1.1.88.22",
    "1.2.840.10008.5.1.4.1.1.88.33",
    // "1.2.840.10008.1.2",
    "1.2.840.10008.1.1",
];
