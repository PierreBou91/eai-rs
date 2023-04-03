use std::{collections::HashMap, path::PathBuf};
use tracing::Level;

/// A Channel is a map of one source to many destinations
pub(crate) type Channel = HashMap<Node, Vec<Node>>;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct Node {
    /// The name of the node
    pub(crate) aet: String,
    /// The IP address of the node
    pub(crate) ip: String,
    /// The port of the node
    pub(crate) port: u16,
    /// Whether the node only accepts uncompressed data
    pub(crate) uncompressed_only: bool,
    /// Whether the node is strict
    pub(crate) strict: bool,
    /// The maximum PDU size
    pub(crate) max_pdu: u32,
    /// The output directory
    pub(crate) out_dir: Option<PathBuf>,
}
#[derive(Debug)]
pub(crate) struct Config {
    /// a map of channels to their id
    pub(crate) channels: HashMap<u64, Channel>,
    /// the log level of the application
    pub(crate) log_level: Level,
}

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
