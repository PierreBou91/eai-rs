use std::{collections::HashMap, net::Ipv4Addr};

use crate::utils::{Channel, Config, Node};

#[allow(clippy::mutable_key_type)]
pub(crate) fn bogus_channel() -> Channel {
    let mut addresses = HashMap::new();
    let node1 = Node::new("NOEUD1in".to_string(), Ipv4Addr::from(0).to_string(), 11112);
    let node2 = Node::new("NOEUD1out".to_string(), "192.168.10.140".to_string(), 11114);
    let aet = node1.aet().clone();
    addresses.insert(node1, vec![node2]);

    Channel {
        name: aet,
        addresses,
        status: Default::default(),
    }
}

#[allow(clippy::mutable_key_type)]
pub(crate) fn bogus_channel2() -> Channel {
    let mut addresses = HashMap::new();
    let node1 = Node::new("NOEUD2in".to_string(), Ipv4Addr::from(0).to_string(), 11113);
    let node2 = Node::new("NOEUD2out".to_string(), "192.168.10.140".to_string(), 11115);
    let aet = node1.aet().clone();
    addresses.insert(node1, vec![node2]);

    Channel {
        name: aet,
        addresses,
        status: Default::default(),
    }
}

#[allow(clippy::mutable_key_type)]
pub(crate) fn bogus_channel3() -> Channel {
    let mut addresses = HashMap::new();
    let node1 = Node::new("NOEUD3in".to_string(), Ipv4Addr::from(0).to_string(), 11113);
    let node2 = Node::new("NOEUD2out".to_string(), "192.168.10.140".to_string(), 11115);
    let aet = node1.aet().clone();
    addresses.insert(node1, vec![node2]);

    Channel {
        name: aet,
        addresses,
        status: Default::default(),
    }
}

#[allow(clippy::mutable_key_type)]
pub(crate) fn bogus_channel4() -> Channel {
    let mut addresses = HashMap::new();
    let node1 = Node::new("NOEUD3in".to_string(), Ipv4Addr::from(0).to_string(), 11116);
    let node2 = Node::new("NOEUD2out".to_string(), "192.168.10.140".to_string(), 11115);
    let aet = node1.aet().clone();
    addresses.insert(node1, vec![node2]);

    Channel {
        name: aet,
        addresses,
        status: Default::default(),
    }
}

#[allow(clippy::mutable_key_type)]
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

pub(crate) fn update_bogus_config(config: &mut Config) {
    config.channels.insert(4, bogus_channel4());
}
