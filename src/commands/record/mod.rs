use crate::{
    convert_audio_file, get_available_cards, is_recording_tool_available, record_audio,
    InsomniaProject,
};
use chrono::{Local, Timelike};
use clap::ArgMatches;
use clap::Clap;
use log::{error, info};
use std::collections::HashMap;
use std::thread::{sleep, spawn};
use std::time::Duration;

/// Record audio files with a specific timing for later analysis (will be produce a lot of data).
#[derive(Clap)]
pub struct RecordCommandOptions {
    /// Select the number of minutes to record in a single file.
    #[clap(long, default_value = "1")]
    duration: u8,

    /// Select the audio card to use for the recording process.
    #[clap(long, default_value = "0")]
    card: u8,

    /// Select the audio device to use for the recording process.
    #[clap(long, default_value = "0")]
    device: u8,

    /// Disable the encoding of the recorded files to mp3 using ffmpeg.
    #[clap(long)]
    no_encoding: bool,

    /// Record the audio file in mono instead of stereo (not all cards support that).
    #[clap(long)]
    mono: bool,
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
    if !is_valid_device_selection(&available_audio_devices, options.card, options.device) {
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
            options.card,
            options.device,
            recording_duration,
            options.mono,
        );
        if file_prefix.is_some() {
            let file_prefix_unwrapped = file_prefix.unwrap();
            info!("The recording {} was finished", file_prefix_unwrapped);
            if should_encode_files {
                spawn(move || {
                    convert_audio_file(file_prefix_unwrapped);
                });
            }
        } else {
            error!(
                "Failed to record an audio stream from card {} and device {}",
                options.card, options.device
            );
        }
    }
}
