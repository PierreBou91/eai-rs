use std::thread;

use bogus::bogus_config;
use color_eyre::eyre::Context;
use tracing::{debug, info, subscriber};
use tracing_subscriber::FmtSubscriber;

use crate::{bogus::update_bogus_config, store_scp::store_scp, utils::State};

pub mod bogus;
pub mod store_scp;
pub mod utils;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // read the config
    let mut config = bogus_config();
    let mut state = State::new();

    subscriber::set_global_default(
        FmtSubscriber::builder()
            .with_max_level(config.log_level)
            .finish(),
    )
    .wrap_err("Error setting the global tracing subscriber")?;

    debug!("{:?}", config);

    let mut handles = vec![];

    let mut bogus_wait = 0;
    loop {
        // Some break condition
        // compare config to state
        if !(config == state) {
            info!("Config has changed, updating the state");

            let nodes = config.nodes();

            for mut node in nodes {
                info!(
                    "Launching the storescp for {} at {}:{}",
                    node.aet, node.ip, node.port
                );
                info!("Node status: {:?}", node.status);
                let handle = thread::spawn(move || store_scp(&mut node).unwrap());
                handles.push(handle);
            }
            // Bogus state update
            state = config.clone();
        }
        // update config
        if bogus_wait == 3 {
            info!("Updating the config");
            update_bogus_config(&mut config);
        }
        bogus_wait += 1;
        // sleep to go easy on the CPU
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // Wait for all threads to complete
    // for handle in handles {
    //     handle.join().unwrap();
    // }
}
