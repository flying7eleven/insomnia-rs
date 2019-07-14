use chrono::{Local, Timelike};
use clap::{load_yaml, App};
use log::{error, info, LevelFilter};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

fn wait_until_full_minute() {
    let last_timestamp = Local::now().naive_local();
    sleep(Duration::from_secs(u64::from(60 - last_timestamp.second())));
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

fn generate_record_command() -> String {
    format!(
        "arecord -D hw:1,0 -d 60 -f S16_LE -r 48000 {}.wav",
        Local::now().naive_local().format("%Y%m%d%H%M%S")
    ) // https://doc.rust-lang.org/std/process/struct.Command.html
}

fn is_recording_tool_available() -> bool {
    let maybe_exit_status = Command::new("arecord").args(&["--version"]).status();

    // if there was an error, we could not execute the command
    if maybe_exit_status.is_err() {
        return false;
    }

    // return the return status of the executed command
    let exit_status = maybe_exit_status.unwrap();
    exit_status.success()
}

fn main() {
    initialize_logging();

    // before we continue we should ensure that the required recording tool is available
    if !is_recording_tool_available() {
        error!("The arecord tool seems not to be available on your computer. Terminating.");
        return;
    }

    // configure the command line parser
    let configuration_parser_config = load_yaml!("cli.yml");
    let matches = App::from_yaml(configuration_parser_config).get_matches();

    // wait until we reached the next full minute
    info!(
        "The current time is {}. We are waiting for the next full minute to start.",
        Local::now().naive_local()
    );
    wait_until_full_minute();

    info!("{}", generate_record_command());
}
