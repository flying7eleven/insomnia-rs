use chrono::{Local, Timelike};
use clap::ArgMatches;
use insomnia::{
    convert_audio_file, get_available_cards, is_recording_tool_available, record_audio,
};
use log::{error, info};
use std::collections::HashMap;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

fn wait_until_full_minute() {
    let last_timestamp = Local::now().naive_local();
    sleep(Duration::from_secs(u64::from(60 - last_timestamp.second())));
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

pub fn run_command_record(argument_matches: &ArgMatches) {
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
    let recording_duration = if argument_matches.is_present("duration") {
        let duration_match = argument_matches.value_of("duration").unwrap();
        60 * duration_match.parse::<u32>().unwrap()
    } else {
        60
    };

    // get the audio card
    let audio_card = if argument_matches.is_present("card") {
        let card_match = argument_matches.value_of("card").unwrap();
        card_match.parse::<u8>().unwrap()
    } else {
        0
    };

    // get the audio device
    let audio_device = if argument_matches.is_present("device") {
        let device_match = argument_matches.value_of("device").unwrap();
        device_match.parse::<u8>().unwrap()
    } else {
        0
    };

    // check if we should encode the files or not
    let should_encode_files = !argument_matches.is_present("no-encoding");
    if !should_encode_files {
        info!("Encoding of the audio files was disabled by a runtime flag");
    }

    // be sure that the audio device selection makes sense
    if !is_valid_device_selection(&available_audio_devices, audio_card, audio_device) {
        panic!("An invalid combination of audio devices was detected.");
    }

    // ensure a sensable recording duration was selected
    if recording_duration < 60 || recording_duration > 3600 {
        panic!("Please select a recording duration between 1 and 60 minutes.");
    }

    // wait until we reached the next full minute
    info!(
        "The current time is {}. We are waiting for the next full minute to start.",
        Local::now().naive_local()
    );
    wait_until_full_minute();

    // record audio files endlessly and convert them to mp3s
    loop {
        let file_prefix = record_audio(
            audio_card,
            audio_device,
            recording_duration,
            argument_matches.is_present("mono"),
        );
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
