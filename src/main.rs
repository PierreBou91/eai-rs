use std::{collections::HashMap, thread};

use bogus::bogus_config;
use color_eyre::eyre::Context;
use tracing::{debug, info, subscriber};
use tracing_subscriber::FmtSubscriber;

use crate::{bogus::update_bogus_config, store_scp::store_scp, utils::State};

pub mod bogus;
pub mod store_scp;
pub mod utils;

// necessary to use the AET as a key in the HashMap
// IMPROVE: find a way to remove this
#[allow(clippy::mutable_key_type)]
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

    let mut handles = HashMap::new();

    let mut bogus_wait = 0;
    loop {
        // Some break condition
        // compare config to state
        if !(config == state) {
            info!("Config has changed, updating the state");

            let actions = state.diff(&config);

            for action in actions {
                match action {
                    utils::Actions::Create(channel) => {
                        info!("Creating channel {}", channel.name);
                        let addresses = channel.addresses.clone();
                        for (mut node, _) in addresses {
                            info!(
                                "Launching the storescp for {} at {}:{}",
                                node.aet(),
                                node.ip,
                                node.port
                            );
                            let aet = node.aet().clone();
                            let handle = thread::spawn(move || node.start_node());
                            handles.insert(aet, handle);
                        }
                    }
                    utils::Actions::Modify(_) => todo!(),
                    utils::Actions::Delete(channel) => {
                        info!("Deleting channel {}", channel.name);
                        let addresses = channel.addresses.clone();
                        for (mut node, _) in addresses {
                            info!(
                                "Stopping the storescp for {} at {}:{}",
                                node.aet(),
                                node.ip,
                                node.port
                            );
                            node.stop_node();
                            let handle = handles.remove(&node.aet().clone()).unwrap();
                            handle.join().unwrap();
                        }
                    }
                }
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
}
