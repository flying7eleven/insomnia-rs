use std::collections::HashMap;
use std::thread::{sleep, spawn, JoinHandle};
use std::time::Duration;

use chrono::{Local, Timelike};
use clap::Clap;
use log::{error, info};

use crate::{
    convert_audio_file, get_available_cards, is_recording_tool_available, record_audio,
    InsomniaProject, RecordingDeviceConfiguration,
};

/// Record audio files with a specific timing for later analysis (will be produce a lot of data).
#[derive(Clap)]
pub struct RecordCommandOptions {
    /// Select the number of minutes to record in a single file.
    #[clap(long, default_value = "1")]
    duration: u8,

    /// Disable the encoding of the recorded files to mp3 using ffmpeg.
    #[clap(long)]
    no_encoding: bool,
}

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

pub fn run_command_record(options: RecordCommandOptions, config: InsomniaProject) {
    // before we continue we should ensure that the required recording tool is available
    if !is_recording_tool_available() {
        error!("The arecord tool seems not to be available on your computer. Terminating.");
        return;
    }

    // ensure that at least one input device is configured
    if config.input.len() < 1 {
        error!("No input device is configured. Terminating.");
        return;
    }

    // get all audio devices of the computer
    let available_audio_devices = get_available_cards()
        .map_err(|_error| panic!("Could not find any suitable audio devices. Terminating."))
        .unwrap();

    // get the recording duration
    let recording_duration = 60 * u32::from(options.duration);

    // check if we should encode the files or not
    let should_encode_files = !options.no_encoding;
    if !should_encode_files {
        info!("Encoding of the audio files was disabled by a runtime flag");
    }

    // be sure that the audio device selection makes sense
    for current_device_key in config.input.keys() {
        let current_device = config.input[current_device_key].clone();
        if !is_valid_device_selection(
            &available_audio_devices,
            current_device.card,
            current_device.device,
        ) {
            panic!(
                "An invalid combination of audio devices (cd:{},{}) was detected.",
                current_device.card, current_device.device
            );
        }
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

    // record audio files endlessly and convert them to mp3s (if requested)
    loop {
        let handles = config
            .input
            .keys()
            .map(|key| {
                let current_device = config.input[key].clone();
                spawn(move || {
                    let file_prefix = record_audio(
                        current_device.card,
                        current_device.device,
                        recording_duration,
                        current_device.mono,
                    );
                    if file_prefix.is_some() {
                        let file_prefix_unwrapped = file_prefix.unwrap();
                        info!(
                            "The recording {} of card {} and device {} was finished",
                            file_prefix_unwrapped, current_device.card, current_device.device
                        );
                    } else {
                        error!(
                            "Failed to record an audio stream from card {} and device {}",
                            current_device.card, current_device.device
                        );
                    }
                })
            })
            .collect::<Vec<JoinHandle<_>>>();

        // wait for the recording threads to finish, should be nearly the same but we better
        // try to sync everything here
        for handle in handles {
            handle.join().unwrap();
        }
        info!("All recording threads finished, continuing for the next run...");
    }
}
