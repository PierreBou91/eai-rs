// use std::{collections::HashMap, net::Ipv4Addr};

// use crate::utils::{Channel, Config, LogLevel, Node};

// pub(crate) fn bogus_channel() -> Channel {
//     let node1 = Node::new("NOEUD1in".to_string(), Ipv4Addr::from(0).to_string(), 11112);
//     let node2 = Node::new("NOEUD1out".to_string(), "192.168.10.140".to_string(), 11114);
//     let aet = node1.aet().clone();

//     Channel {
//         name: aet,
//         source: node1,
//         destinations: vec![node2],
//         status: Default::default(),
//     }
// }

// pub(crate) fn bogus_config() -> Config {
//     let mut config = Config {
//         channels: HashMap::new(),
//         log_level: LogLevel::Info,
//     };
//     config.channels.insert(1, bogus_channel());
//     config
// }
