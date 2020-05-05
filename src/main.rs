use chrono::Local;
use clap::{crate_authors, crate_description, crate_version, Clap};
use log::{error, LevelFilter};

use schlaflosigkeit::commands::annotate::{run_command_annotate, AnnotateCommandOptions};
use schlaflosigkeit::commands::config::{run_command_config, ConfigCommandOptions};
use schlaflosigkeit::commands::record::{run_command_record, RecordCommandOptions};
use schlaflosigkeit::{InsomniaProject, RecordingDeviceConfiguration};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
struct Opts {
    /// The sub-command which should be executed.
    #[clap(subcommand)]
    subcmd: SubCommand,

    /// The project file which defines how to process data. The used information depend on the used
    /// sub-command.
    #[clap(index = 1)]
    project: String,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
    Config(ConfigCommandOptions),

    #[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
    Record(RecordCommandOptions),

    #[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
    Annotate(AnnotateCommandOptions),
}

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

    // parse the options provided by the user
    let opts: Opts = Opts::parse();

    // try to read the configuration file
    let configuration: InsomniaProject = match File::open(opts.project) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content);
            match toml::from_str(content.as_str()) {
                Ok(object) => object,
                Err(error) => {
                    error!("Could not parse the project file. The error was: {}", error);
                    return;
                }
            }
        }
        Err(error) => {
            error!("Could not read the project file. The error was: {}", error);
            return;
        }
    };

    // check which subcommand should be executed and call it
    match opts.subcmd {
        SubCommand::Annotate(suboptions) => run_command_annotate(suboptions, configuration),
        SubCommand::Config(suboptions) => run_command_config(suboptions, configuration),
        SubCommand::Record(suboptions) => run_command_record(suboptions, configuration),
    }
}
