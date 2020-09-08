use core::fmt;
use std::collections::HashMap;
use std::env::current_dir;
use std::error;
use std::process::{Command, Stdio};

use chrono::Local;
use log::{debug, error, info};
use regex::bytes::Regex;
use serde::{Deserialize, Serialize};

use lazy_static::lazy_static;
use std::path::Path;

pub mod annotation;
pub mod commands;

lazy_static! {
    static ref CARD_AND_DEVICES_REGEX: Regex = Regex::new(r"card (\d*):.*device (\d*):").unwrap();
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct RecordingDeviceConfiguration {
    #[serde(default = "RecordingDeviceConfiguration::default_card")]
    pub card: u8,

    #[serde(default = "RecordingDeviceConfiguration::default_device")]
    pub device: u8,

    #[serde(default = "RecordingDeviceConfiguration::default_mono")]
    pub mono: bool,
}

impl RecordingDeviceConfiguration {
    fn default_device() -> u8 {
        0
    }

    fn default_card() -> u8 {
        0
    }

    fn default_mono() -> bool {
        false
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct InsomniaProject {
    #[serde(default = "InsomniaProject::default_data_directory")]
    pub data_directory: String,

    #[serde(default = "InsomniaProject::default_input")]
    pub input: HashMap<String, RecordingDeviceConfiguration>,
}

impl InsomniaProject {
    fn default_data_directory() -> String {
        match current_dir() {
            Ok(current_dir) => match current_dir.to_str() {
                Some(current_dir_str) => current_dir_str.to_string(),
                None => "".to_string(),
            },
            Err(_) => "".to_string(),
        }
    }

    fn default_input() -> HashMap<String, RecordingDeviceConfiguration> {
        let mut default_device = HashMap::new();
        default_device.insert(
            "default_device".to_string(),
            RecordingDeviceConfiguration {
                card: 0,
                device: 0,
                mono: false,
            },
        );
        default_device
    }
}

#[derive(Debug, Clone)]
pub struct AudioDeviceError;

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
/// use insomnia::get_available_cards;;
///
/// let devices = get_available_cards();
///
/// if devices.is_err() {
///   panic!("Could not get the available audio devices");
/// }
///
/// for device in devices.unwrap() {
///   println!("Found audio device: {:?}", device);
/// }
/// ```
pub fn get_available_cards() -> Result<HashMap<u8, (u8, u8)>, AudioDeviceError> {
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

pub fn record_audio(
    card: u8,
    device: u8,
    duration_in_seconds: u32,
    record_mono: bool,
    output_folder: String,
) -> Option<String> {
    let file_prefix = Local::now()
        .naive_local()
        .format("%Y%m%d%H%M%S_%f")
        .to_string();

    let output_file_pattern = format!("{}_c{:02}d{:02}.wav", file_prefix, card, device);
    let output_file = Path::new(&output_folder).join(Path::new(&output_file_pattern));
    let mut record_command = Command::new("arecord");
    record_command
        .arg(format!("-Dhw:{},{}", card, device))
        .arg(format!("-d{}", duration_in_seconds))
        .arg("-fS16_LE")
        .arg("-r44100")
        .arg(output_file.to_str().unwrap())
        .stderr(Stdio::null())
        .stdout(Stdio::null());

    // ensure the right flag (mono or stereo) is set
    if record_mono {
        record_command.arg("-c1");
    } else {
        record_command.arg("-c2");
    }

    // now we can start the program and check its return status
    let record_status = record_command.status();
    if record_status.is_ok() && record_status.unwrap().success() {
        return Some(file_prefix);
    }

    None
}

pub fn convert_audio_file(file_prefix: String) {
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

pub fn is_recording_tool_available() -> bool {
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
