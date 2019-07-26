use chrono::{Local, Timelike};
use clap::{crate_authors, crate_description, crate_name, crate_version, load_yaml, App};
use lazy_static::lazy_static;
use log::{debug, error, info, LevelFilter};
use regex::bytes::Regex;
use std::collections::HashMap;
use std::fmt;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;
use std::{error, thread};

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

fn record_audio(card: u8, device: u8, duration_in_seconds: u32) -> Option<String> {
    let file_prefix = Local::now()
        .naive_local()
        .format("%Y%m%d%H%M%S")
        .to_string();

    let record_status = Command::new("arecord")
        .arg(format!("-Dhw:{},{}", card, device))
        .arg(format!("-d{}", duration_in_seconds))
        .arg("-fS16_LE")
        .arg("-c2")
        .arg("-r48000")
        .arg(format!("{}.wav", file_prefix))
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status();

    if record_status.is_ok() && record_status.unwrap().success() {
        return Some(file_prefix);
    }

    None
}

fn convert_audio_file(file_prefix: String) {
    info!("Converting {}.wav to {}.mp3", file_prefix, file_prefix);
    let convert_status = Command::new("ffmpeg")
        .arg("-i")
        .arg(format!("{}.wav", file_prefix))
        .arg(format!("{}.mp3", file_prefix))
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status();

    // if the conversion was successful, we can remove the old record of the audio file
    if convert_status.is_ok() && convert_status.unwrap().success() {
        debug!(
            "File conversion successful, removing old {}.wav file",
            file_prefix
        );
        let _remove_status = Command::new("rm")
            .arg("-rf")
            .arg(format!("{}.wav", file_prefix))
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .spawn();
    }
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

    // if we do not have found any audio devices, also exit with an error
    if device_list.is_empty() {
        return Err(AudioDeviceError);
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

fn is_valid_device_selection(
    available_audio_devices: &HashMap<u8, (u8, u8)>,
    audio_card: u8,
    audio_device: u8,
) -> bool {
    let current_audio_card_device_tuple = available_audio_devices.get(&audio_card);
    if current_audio_card_device_tuple.is_some() {
        let (_, cur_device) = current_audio_card_device_tuple.unwrap();
        if *cur_device == audio_device {
            return true;
        }
    }
    false
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

    // before we continue we should ensure that the required recording tool is available
    if !is_recording_tool_available() {
        error!("The arecord tool seems not to be available on your computer. Terminating.");
        return;
    }

    // get all audio devices of the computer
    let available_audio_devices = get_available_cards()
        .map_err(|_error| panic!("Could not find any suitable audio devices. Terminating."))
        .unwrap();

    // get the recording duration
    let recording_duration = if matches.is_present("duration") {
        let duration_match = matches.value_of("duration").unwrap();
        60 * duration_match.parse::<u32>().unwrap()
    } else {
        60
    };

    // get the audio card
    let audio_card = if matches.is_present("card") {
        let card_match = matches.value_of("card").unwrap();
        card_match.parse::<u8>().unwrap()
    } else {
        0
    };

    // get the audio device
    let audio_device = if matches.is_present("device") {
        let device_match = matches.value_of("device").unwrap();
        device_match.parse::<u8>().unwrap()
    } else {
        0
    };

    // check if we should encode the files or not
    let should_encode_files = !matches.is_present("no-encoding");
    if !should_encode_files {
        info!("Encoding of the audio files was disabled by a runtime flag");
    }

    // be sure that the audio device selection makes sense
    if !is_valid_device_selection(&available_audio_devices, audio_card, audio_device) {
        panic!("An invalid combination of audio devices was detected.");
    }

    // ensure a sensable recording duration was selected
    if recording_duration < 60 || recording_duration > 600 {
        panic!("Please select a recording duration between 1 and 10 minutes.");
    }

    // wait until we reached the next full minute
    info!(
        "The current time is {}. We are waiting for the next full minute to start.",
        Local::now().naive_local()
    );
    wait_until_full_minute();

    // record audio files endlessly and convert them to mp3s
    loop {
        let file_prefix = record_audio(audio_card, audio_device, recording_duration);
        if file_prefix.is_some() {
            let file_prefix_unwrapped = file_prefix.unwrap();
            info!("The recording {} was finished", file_prefix_unwrapped);
            if should_encode_files {
                thread::spawn(move || {
                    convert_audio_file(file_prefix_unwrapped);
                });
            }
        } else {
            error!("Failed to record an audio stream.");
        }
    }
}
