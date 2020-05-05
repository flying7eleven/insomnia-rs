use chrono::Local;
use clap::{crate_authors, crate_description, crate_version, Clap};
use log::{error, LevelFilter};

use schlaflosigkeit::commands::annotate::{run_command_annotate, AnnotateCommandOptions};
use schlaflosigkeit::commands::config::{run_command_config, ConfigCommandOptions};
use schlaflosigkeit::commands::record::{run_command_record, RecordCommandOptions};

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

    // configure the command line parser
    let opts: Opts = Opts::parse();

    // check which subcommand should be executed and call it
    match opts.subcmd {
        SubCommand::Annotate(suboptions) => run_command_annotate(suboptions),
        SubCommand::Config(suboptions) => run_command_config(suboptions),
        SubCommand::Record(suboptions) => run_command_record(suboptions),
    }
}
