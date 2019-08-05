use chrono::Local;
use clap::{crate_authors, crate_description, crate_name, crate_version, load_yaml, App};
use log::{error, LevelFilter};
use schlaflosigkeit::annotate::run_command_annotate;
use schlaflosigkeit::record::run_command_record;
use schlaflosigkeit::denoise::run_command_denoise;

fn initialize_logging() {
    // configure the logging framework and set the corresponding log level
    let logging_framework = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply();

    // ensure the logging framework was successfully initialized
    if logging_framework.is_err() {
        panic!("Could not initialize the logging framework. Terminating!");
    }
}

fn main() {
    initialize_logging();

    // configure the command line parser
    let configuration_parser_config = load_yaml!("cli.yml");
    let matches = App::from_yaml(configuration_parser_config)
        .author(crate_authors!())
        .version(crate_version!())
        .name(crate_name!())
        .about(crate_description!())
        .get_matches();

    // check which subcommand should be executed and call it
    if let Some(matches) = matches.subcommand_matches("record") {
        run_command_record(matches);
    } else if let Some(matches) = matches.subcommand_matches("annotate") {
        run_command_annotate(matches);
    } else if let Some(matches) = matches.subcommand_matches("denoise") {
        run_command_denoise(matches);
    } else {
        error!("No known subcommand was selected. Please refer to the help for information about how to use this application.");
    }
}
