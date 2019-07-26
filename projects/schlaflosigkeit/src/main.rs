use chrono::{Local, Timelike};
use clap::{crate_authors, crate_description, crate_name, crate_version, load_yaml, App};
use insomnia::{
    convert_audio_file, get_available_cards, is_recording_tool_available, record_audio,
};
use log::{error, info, LevelFilter};
use std::collections::HashMap;
use std::thread;
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
