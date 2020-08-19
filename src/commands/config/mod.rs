use clap::Clap;
use log::warn;

use crate::InsomniaProject;

/// A sub-command for showing configuration options and storing an example configuration
#[derive(Clap)]
pub struct ConfigCommandOptions {
    ///
    #[clap(long)]
    save_sample: bool,
}

pub fn run_command_config(options: ConfigCommandOptions, config: InsomniaProject) {
    if options.save_sample {
        warn!("The save option is currently not implemented!");
        return;
    }

    // just print the information from the configuration file
    println!("[*] Data directory:\t\t{}", config.data_directory);
    println!("[*] Input device count:\t\t{}", config.input.len());
    for current_input_device_name in config.input.keys() {
        println!("    [-] Defined name:\t\t{}", current_input_device_name);
        println!(
            "        [-] Card:\t\t{}",
            config.input[current_input_device_name].card
        );
        println!(
            "        [-] Device:\t\t{}",
            config.input[current_input_device_name].device
        );
        println!(
            "        [-] Mono:\t\t{}",
            config.input[current_input_device_name].mono
        );
    }
}
