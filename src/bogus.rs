use std::{collections::HashMap, path::PathBuf, net::Ipv4Addr};

use crate::utils::{Channel, Config, Node};

pub(crate) fn bogus_channel() -> Channel {
    let mut channel = HashMap::new();
    let node1 = Node {
        aet: "NOEUD1".to_string(),
        ip: Ipv4Addr::from(0).to_string(),
        port: 11112,
        uncompressed_only: false,
        max_pdu: 16384,
        strict: false,
        out_dir: Some(PathBuf::from(".")),
    };
    let node2 = Node {
        aet: "NOEUD1.2".to_string(),
        ip: "192.168.10.140".to_string(),
        port: 11113,
        max_pdu: 16384,
        uncompressed_only: false,
        strict: false,
        out_dir: Some(PathBuf::from(".")),
    };
    channel.insert(node1, vec![node2]);
    channel
}

pub(crate) fn bogus_channel2() -> Channel {
    let mut channel = HashMap::new();
    let node1 = Node {
        aet: "NOEUD2".to_string(),
        ip: Ipv4Addr::from(0).to_string(),
        port: 11114,
        uncompressed_only: false,
        max_pdu: 16384,
        strict: false,
        out_dir: Some(PathBuf::from(".")),
    };
    let node2 = Node {
        aet: "NOEUD2.2".to_string(),
        ip: "192.168.10.140".to_string(),
        port: 11115,
        max_pdu: 16384,
        uncompressed_only: false,
        strict: false,
        out_dir: Some(PathBuf::from(".")),
    };
    channel.insert(node1, vec![node2]);
    channel
}

pub(crate) fn bogus_channel3() -> Channel {
    let mut channel = HashMap::new();
    let node1 = Node {
        aet: "NOEUD3".to_string(),
        ip: Ipv4Addr::from(0).to_string(),
        port: 11114,
        uncompressed_only: false,
        max_pdu: 16384,
        strict: false,
        out_dir: Some(PathBuf::from(".")),
    };
    let node2 = Node {
        aet: "NOEUD3.2".to_string(),
        ip: "192.168.10.140".to_string(),
        port: 11115,
        max_pdu: 16384,
        uncompressed_only: false,
        strict: false,
        out_dir: Some(PathBuf::from(".")),
    };
    channel.insert(node1, vec![node2]);
    channel
}

pub(crate) fn bogus_config() -> Config {
    let mut config = Config {
        channels: HashMap::new(),
        log_level: tracing::Level::INFO,
    };
    config.channels.insert(1, bogus_channel());
    config.channels.insert(2, bogus_channel2());
    config.channels.insert(3, bogus_channel3());
    config
}
