use std::thread::{self};

use bogus::bogus_config;
use color_eyre::eyre::Context;
use tracing::{debug, info, subscriber};
use tracing_subscriber::FmtSubscriber;

use crate::{store_scp::store_scp, utils::Node};

pub mod bogus;
pub mod store_scp;
pub mod utils;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // read the config
    let config = bogus_config();

    subscriber::set_global_default(
        FmtSubscriber::builder()
            .with_max_level(config.log_level)
            .finish(),
    )
    .wrap_err("Error setting the global tracing subscriber")?;

    debug!("{:?}", config);

    let mut handles = vec![];

    let nodes: Vec<Node> = config
        .channels
        .values()
        .map(|v| v.keys().next().unwrap().clone())
        .collect();

    for node in nodes {
        info!("Launching the store scp for {}", node.aet);
        let handle =
            thread::spawn(move || store_scp(&node).wrap_err("Error launching the store_scp"));
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap()?;
    }

    // test the connection to the peers (echoscu)

    // launch a dicom node

    Ok(())
}
