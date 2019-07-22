use chrono::{Local, Timelike};
use clap::{crate_authors, crate_description, crate_name, crate_version, load_yaml, App};
use lazy_static::lazy_static;
use log::{debug, error, info, LevelFilter};
use regex::bytes::Regex;
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Clone)]
struct AudioDeviceError;

impl fmt::Display for AudioDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown audio device error")
    }
}

impl error::Error for AudioDeviceError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

lazy_static! {
    static ref CARD_AND_DEVICES_REGEX: Regex = Regex::new(r"card (\d*):.*device (\d*):").unwrap();
}

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

/// Get a list of valid audio cards and their devices.
///
/// # Errors
/// TODO
///
/// # Example
///
/// Simple way of using this method:
///
/// ```
/// let devices = get_available_cards();
///
/// if devices.is_err() {
///   panic!("Could not get the available audio devices");
/// }
///
/// for device in devices.unwrap() {
///   ...
/// }
/// ```
fn get_available_cards() -> Result<HashMap<u8, (u8, u8)>, AudioDeviceError> {
    let maybe_list_devices_output = Command::new("arecord").args(&["-l"]).output();

    //
    if maybe_list_devices_output.is_err() {
        error!("Could not get list of audio devices!");
        return Err(AudioDeviceError);
    }

    //
    let list_devices_output = maybe_list_devices_output.unwrap();
    let actual_text_output = String::from_utf8_lossy(&list_devices_output.stdout).to_string();
    let mut device_list = HashMap::new();

    //
    for cap in CARD_AND_DEVICES_REGEX.captures_iter(actual_text_output.as_bytes()) {
        let card_id: u8 = String::from_utf8_lossy(&cap[1]).parse().unwrap();
        let device_id: u8 = String::from_utf8_lossy(&cap[2]).parse().unwrap();
        debug!("Found audio card {} with device {}", card_id, device_id);
        device_list.insert(card_id, (card_id, device_id));
    }

    Ok(device_list)
}

fn is_recording_tool_available() -> bool {
    let maybe_exit_status = Command::new("arecord")
        .args(&["--version"])
        .stdout(Stdio::null())
        .status();

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

    let available_audio_devices = get_available_cards();

    // configure the command line parser
    let configuration_parser_config = load_yaml!("cli.yml");
    let matches = App::from_yaml(configuration_parser_config)
        .author(crate_authors!())
        .version(crate_version!())
        .name(crate_name!())
        .about(crate_description!())
        .get_matches();

    // wait until we reached the next full minute
    info!(
        "The current time is {}. We are waiting for the next full minute to start.",
        Local::now().naive_local()
    );
    wait_until_full_minute();

    info!("{}", generate_record_command());
}
