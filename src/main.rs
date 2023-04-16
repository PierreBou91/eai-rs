use std::{collections::HashMap, path::Path, sync::Arc, thread};

use color_eyre::eyre::Context;
use tracing::{debug, info, subscriber};
use tracing_subscriber::FmtSubscriber;

use crate::{
    store_scp::store_scp,
    utils::{Config, State},
};

pub mod bogus;
pub mod store_scp;
pub mod utils;

const CONFIG_FILE: &str = "config.json";

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let config_path = Path::new(CONFIG_FILE);

    let mut config = Config::from_json_file(Path::new(config_path))?;
    let mut state = State::new();

    subscriber::set_global_default(
        FmtSubscriber::builder()
            .with_max_level(config.log_level.to_tracing_level())
            .finish(),
    )
    .wrap_err("Error setting the global tracing subscriber")?;

    match Config::to_json_file(&config, Path::new("config.json")) {
        Ok(_) => info!("Config file created"),
        Err(e) => info!("Error creating the config file: {}", e),
    }

    debug!("{:?}", config);

    let mut handles = HashMap::new();

    loop {
        if !(config == state) {
            info!("Config has changed, updating the state");

            {
                let actions = state.diff(&config);

                for action in actions {
                    match action {
                        utils::Actions::Create(channel) => {
                            let channel = Arc::new(channel);
                            info!("Creating channel {}", channel.name);
                            info!(
                                "Launching the storescp for {} at {}:{}",
                                channel.source.aet, channel.source.ip, channel.source.port
                            );
                            let handle = thread::spawn(|| channel.source.start_node());
                            handles.insert(channel.name, handle);
                        }
                        utils::Actions::Modify(_) => todo!(),
                        utils::Actions::Delete(mut channel) => {
                            let channel = Arc::new(channel);
                            info!("Deleting channel {}", channel.name);
                            channel.source.stop_node();
                            handles.remove(&channel.name).unwrap();
                        }
                    }
                }
            }

            state = config.clone();
        }
        // update config
        config = Config::from_json_file(Path::new(config_path))?;

        // sleep to go easy on the CPU
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
