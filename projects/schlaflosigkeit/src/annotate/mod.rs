use chrono::{Duration as OldDuration, TimeZone, Timelike, Utc};
use clap::ArgMatches;
use insomnia::annotation::WaveMetaReader;
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use std::fs::{read_dir, OpenOptions};
use std::io::Write;
use std::ops::Add;

lazy_static! {
    static ref CORRECT_FILE_NAME_REGEX: Regex =
        Regex::new(r".*(\d{4})(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})\.wav").unwrap();
}

pub fn run_command_annotate(argument_matches: &ArgMatches) {
    // ensure ta input folder was specified
    if !argument_matches.is_present("input_folder") {
        error!("No input folder specified. Cannot process files for annotation label generation.");
        return;
    }

    // ensure and output file was specified
    if !argument_matches.is_present("output_file") {
        error!("No output file for the labels specified. Cannot process files for annotation label generation.");
        return;
    }

    //
    let mut start_label: f64 = 0.0;
    let mut label_file = match OpenOptions::new()
        .append(true)
        .create(true)
        .open(argument_matches.value_of("output_file").unwrap())
    {
        Ok(file) => file,
        Err(error) => {
            error!(
                "Could not open output file. The error was: {}",
                error.to_string()
            );
            return;
        }
    };

    // check if we should use range markers or not
    let range_mode = argument_matches.is_present("range");

    //
    let no_date = argument_matches.is_present("no-date");

    // loop through all found files and try to process them
    for maybe_audio_file_path in
        read_dir(argument_matches.value_of("input_folder").unwrap()).unwrap()
    {
        let audio_file_path_obj = maybe_audio_file_path.unwrap().path();
        let audio_file_path = audio_file_path_obj.to_str().unwrap();

        // ensure the skip all files which do not match the expected pattern
        if !CORRECT_FILE_NAME_REGEX.is_match(audio_file_path) {
            info!(
                "Skipping {} since the filename did not match the expected pattern",
                audio_file_path
            );
            continue;
        }

        // even if there should be one match, try to "loop" through it
        for cap in CORRECT_FILE_NAME_REGEX.captures_iter(audio_file_path) {
            let maybe_meta_reader = WaveMetaReader::from_file(audio_file_path);
            if maybe_meta_reader.is_err() {
                error!(
                    "Could not read the meta information of the file. The error was: {}",
                    maybe_meta_reader.err().unwrap().to_string()
                );
                continue;
            }
            let meta_reader = maybe_meta_reader.unwrap();
            let duration_in_seconds = meta_reader.get_duration();

            let end_label = if range_mode {
                start_label + duration_in_seconds
            } else {
                start_label
            };

            let duration_label = if range_mode {
                let current_timestamp_str = format!(
                    "{:02}.{:02}.{:04} {:02}:{:02}:{:02}",
                    &cap[3], &cap[2], &cap[1], &cap[4], &cap[5], &cap[6],
                );
                let initial_parsed_start_datetime = Utc
                    .datetime_from_str(current_timestamp_str.as_str(), "%d.%m.%Y %H:%M:%S")
                    .unwrap();
                let new_end_date = initial_parsed_start_datetime
                    .add(OldDuration::seconds(duration_in_seconds as i64));

                format!(
                    "{:02}:{:02}:{:02} - {:02}:{:02}:{:02}",
                    &cap[4],
                    &cap[5],
                    &cap[6],
                    new_end_date.hour(),
                    new_end_date.minute(),
                    new_end_date.second()
                )
            } else {
                format!("{}:{}:{}", &cap[4], &cap[5], &cap[6])
            };

            let label_line = if no_date {
                format!("{:.2}\t{:.2}\t{}\n", start_label, end_label, duration_label)
            } else {
                format!(
                    "{:.2}\t{:.2}\t{}.{}.{} {}\n",
                    start_label, end_label, &cap[3], &cap[2], &cap[1], duration_label
                )
            };

            let _ = write!(&mut label_file, "{}", label_line);

            start_label += duration_in_seconds;
        }
    }
}
