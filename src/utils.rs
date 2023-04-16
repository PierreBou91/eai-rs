use core::hash::Hash;
use std::{
    collections::HashMap,
    fmt,
    fs::File,
    hash::Hasher,
    io::{BufReader, BufWriter, Error},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::store_scp;

/// A Channel describes a flow of data between an origin node which
/// is the key of the addresses hashmap and a list of destination
/// nodes.
/// The program will try to launch a C-STORE scp with the origin
/// node (if the status is set to Started) that forwards the data
/// to the destination nodes.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct Channel {
    pub(crate) name: String,
    pub(crate) source: Node,
    pub(crate) destinations: Vec<Node>,
    pub(crate) status: Status,
}

/// A Node is a data structure representing a dicom destination
/// At the moment of this writing it mainly represents an abstraction
/// over a C-STORE scp.
/// It is used to represent both the source and the destination of a
/// channel and does not discriminate between the two.
/// IMPROVE: There might be a better way to do this abstraction
#[derive(Serialize, Deserialize)]
pub(crate) struct Node {
    /// The AET of the node
    pub(crate) aet: String, // This CANNOT be mutable since it is used as a key in a HashMap
    /// The IP address of the node
    pub(crate) ip: String,
    /// The port of the node
    pub(crate) port: u16,
    /// Whether the node only accepts uncompressed data
    pub(crate) uncompressed_only: bool,
    /// The maximum PDU size
    pub(crate) max_pdu: u32,
    /// Whether the node enforce PDU size
    pub(crate) strict: bool,
    /// The output directory if relevant
    pub(crate) out_dir: Option<PathBuf>,
    /// The node's status
    pub(crate) status: Status,
    /// The node's shutdown signal, this is set to true when the
    /// node should be shutdown
    pub(crate) shutdown_signal: AtomicBool,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("aet", &self.aet)
            .field("ip", &self.ip)
            .field("port", &self.port)
            .field("uncompressed_only", &self.uncompressed_only)
            .field("max_pdu", &self.max_pdu)
            .field("strict", &self.strict)
            .field("out_dir", &self.out_dir)
            .field(
                "shutdown_signal",
                &self.shutdown_signal.load(Ordering::SeqCst),
            )
            .finish()
    }
}

impl Eq for Node {}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.aet == other.aet
            && self.ip == other.ip
            && self.port == other.port
            && self.uncompressed_only == other.uncompressed_only
            && self.max_pdu == other.max_pdu
            && self.strict == other.strict
            && self.out_dir == other.out_dir
            && self.status == other.status
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Self {
            aet: self.aet.clone(),
            ip: self.ip.clone(),
            port: self.port,
            uncompressed_only: self.uncompressed_only,
            max_pdu: self.max_pdu,
            strict: self.strict,
            out_dir: self.out_dir.clone(),
            status: self.status.clone(),
            shutdown_signal: AtomicBool::new(self.shutdown_signal.load(Ordering::SeqCst)),
        }
    }
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.aet.hash(state);
    }
}

impl Node {
    pub(crate) fn start_node(&mut self) {
        self.shutdown_signal.store(false, Ordering::SeqCst);
        self.status = Status::Started;
        store_scp(self).unwrap();
    }

    pub(crate) fn stop_node(&mut self) {
        info!("Stopping node {}...", self.aet);
        self.shutdown_signal.store(true, Ordering::SeqCst);
        self.status = Status::Stopped;
    }

    pub(crate) fn aet(&self) -> &String {
        &self.aet
    }
}

#[derive(Default, Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub(crate) enum Status {
    Started,
    #[default]
    Stopped,
}

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) enum LogLevel {
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub(crate) fn to_tracing_level(&self) -> tracing::Level {
        match self {
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct Config {
    /// A map of channels to their id
    pub(crate) channels: HashMap<u64, Channel>,
    /// The log level of the application
    pub(crate) log_level: LogLevel,
}

impl Config {
    pub(crate) fn new() -> Self {
        Self {
            channels: HashMap::new(),
            log_level: LogLevel::Info,
        }
    }

    pub(crate) fn from_json_file(path: &Path) -> Result<Self, Error> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return Ok(Self::new()),
        };
        let reader = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)?;
        Ok(config)
    }

    pub(crate) fn to_json_file(&self, path: &Path) -> Result<(), Error> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    /// Returns a list of actions that need to be performed.
    /// It is important to use this method on the config passing
    /// the state and not the other way around.
    pub(crate) fn diff<'a>(&'a self, config: &'a Config) -> Vec<Actions<'a>> {
        let mut actions = Vec::new();
        for (id, channel) in self.channels.iter() {
            if let Some(other_channel) = config.channels.get(id) {
                if channel != other_channel {
                    actions.push(Actions::Modify(channel));
                }
            } else {
                actions.push(Actions::Delete(channel));
            }
        }
        for (id, channel) in config.channels.iter() {
            if !self.channels.contains_key(id) {
                actions.push(Actions::Create(channel));
            }
        }
        actions
    }
}

#[derive(Debug)]
pub(crate) enum Actions<'a> {
    Create(&'a Channel),
    Modify(&'a Channel),
    Delete(&'a Channel),
}

pub(crate) type State = Config;

pub(crate) static ABSTRACT_SYNTAXES: &[&str] = &[
    "1.2.840.10008.5.1.4.1.1.2",      // CT Image Storage
    "1.2.840.10008.5.1.4.1.1.2.1",    // Enhanced CT Image Storage
    "1.2.840.10008.5.1.4.1.1.9",      // Standalone Curve Storage (Retired)
    "1.2.840.10008.5.1.4.1.1.8",      // Standalone Overlay Storage (Retired)
    "1.2.840.10008.5.1.4.1.1.7",      // Secondary Capture Image Storage
    "1.2.840.10008.5.1.4.1.1.6",      // Ultrasound Image Storage (Retired)
    "1.2.840.10008.5.1.4.1.1.5",      // Nuclear Medicine Image Storage (Retired)
    "1.2.840.10008.5.1.4.1.1.4",      // MR Image Storage
    "1.2.840.10008.5.1.4.1.1.4.1",    // Enhanced MR Image Storage
    "1.2.840.10008.5.1.4.1.1.4.2",    // MR Spectroscopy Storage
    "1.2.840.10008.5.1.4.1.1.4.3",    // Enhanced MR Color Image Storage
    "1.2.840.10008.5.1.4.1.1.3",      // Ultrasound Multi-frame Image Storage (Retired)
    "1.2.840.10008.5.1.4.1.1.1",      // Computed Radiography Image Storage
    "1.2.840.10008.5.1.4.1.1.1.1",    // Digital X-Ray Image Storage - For Presentation
    "1.2.840.10008.5.1.4.1.1.1.1.1",  // Digital X-Ray Image Storage - For Processing
    "1.2.840.10008.5.1.4.1.1.104.1",  // Encapsulated PDF Storage
    "1.2.840.10008.5.1.4.1.1.104.2",  // Encapsulated CDA Storage
    "1.2.840.10008.5.1.4.1.1.104.3",  // Encapsulated STL Storage
    "1.2.840.10008.5.1.4.1.1.11.1",   // Grayscale Softcopy Presentation State Storage
    "1.2.840.10008.5.1.4.1.1.128",    // Positron Emission Tomography Image Storage
    "1.2.840.10008.5.1.4.1.1.13.1.3", // Breast Tomosynthesis Image Storage
    "1.2.840.10008.5.1.4.1.1.13.1.4", // Breast Projection X-Ray Image Storage - For Presentation
    "1.2.840.10008.5.1.4.1.1.13.1.5", // Breast Projection X-Ray Image Storage - For Processing
    "1.2.840.10008.5.1.4.1.1.130",    // Enhanced PET Image Storage
    "1.2.840.10008.5.1.4.1.1.481.1",  // RT Image Storage
    "1.2.840.10008.5.1.4.1.1.20",     // Nuclear Medicine Image Storage
    "1.2.840.10008.5.1.4.1.1.3.1",    // Ultrasound Multi-frame Image Storage
    "1.2.840.10008.5.1.4.1.1.7.1",    // Multi-frame Single Bit Secondary Capture Image Storage
    "1.2.840.10008.5.1.4.1.1.7.2",    // Multi-frame Grayscale Byte Secondary Capture Image Storage
    "1.2.840.10008.5.1.4.1.1.7.3",    // Multi-frame Grayscale Word Secondary Capture Image Storage
    "1.2.840.10008.5.1.4.1.1.7.4",    // Multi-frame True Color Secondary Capture Image Storage
    "1.2.840.10008.5.1.4.1.1.88.11",  // Basic Text SR Storage
    "1.2.840.10008.5.1.4.1.1.88.22",  // Enhanced SR Storage
    "1.2.840.10008.5.1.4.1.1.88.33",  // Comprehensive SR Storage
    "1.2.840.10008.1.1",              // Verification SOP Class
];
