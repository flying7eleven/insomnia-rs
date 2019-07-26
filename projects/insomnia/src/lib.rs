use core::fmt;
use lazy_static::lazy_static;
use log::{debug, error};
use regex::bytes::Regex;
use std::collections::HashMap;
use std::error;
use std::process::Command;

lazy_static! {
    static ref CARD_AND_DEVICES_REGEX: Regex = Regex::new(r"card (\d*):.*device (\d*):").unwrap();
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
