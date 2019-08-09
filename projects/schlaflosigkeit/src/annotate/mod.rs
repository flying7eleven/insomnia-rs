use chrono::{Duration as OldDuration, NaiveDate, NaiveDateTime, TimeZone, Timelike, Utc};
use clap::ArgMatches;
use insomnia::annotation::{FileAnnotator, WaveMetaReader};
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use std::borrow::Borrow;
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
    let add_sub_markers = argument_matches.is_present("add-sub-markers");

    // loop through all found files and try to process them
    let mut ordered_file_list: Vec<String> = vec![];
    for maybe_audio_file_path in
        read_dir(argument_matches.value_of("input_folder").unwrap()).unwrap()
    {
        let audio_file_path_obj = maybe_audio_file_path.unwrap().path();
        let audio_file_path = audio_file_path_obj.to_str().unwrap();
        ordered_file_list.push(audio_file_path.to_string())
    }
    ordered_file_list.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut file_start_time = 0;

    // loop through all found files and try to process them
    for audio_file_path in ordered_file_list {
        // ensure the skip all files which do not match the expected pattern
        if !CORRECT_FILE_NAME_REGEX.is_match(audio_file_path.borrow()) {
            info!(
                "Skipping {} since the filename did not match the expected pattern",
                audio_file_path
            );
            continue;
        }

        // even if there should be one match, try to "loop" through it
        for cap in CORRECT_FILE_NAME_REGEX.captures_iter(audio_file_path.borrow()) {
            let current_timestamp_str = format!(
                "{:02}.{:02}.{:04} {:02}:{:02}:{:02}",
                &cap[3], &cap[2], &cap[1], &cap[4], &cap[5], &cap[6],
            );

            let initial_parsed_start_datetime = Utc
                .datetime_from_str(current_timestamp_str.as_str(), "%d.%m.%Y %H:%M:%S")
                .unwrap()
                .naive_utc();

            let maybe_file_annotator = FileAnnotator::from(
                &audio_file_path,
                initial_parsed_start_datetime,
                file_start_time as u64,
                add_sub_markers,
                range_mode,
            );
            if maybe_file_annotator.is_none() {
                error!("Could not get a file annotator for {}", audio_file_path);
                continue;
            }
            let file_annotator = maybe_file_annotator.unwrap();
            let max_labels = file_annotator.get_max_labels();

            //
            file_start_time = file_annotator.get_end_time();

            //
            for current_label in file_annotator.take(max_labels) {
                let _ = write!(&mut label_file, "{}", current_label.get_label_line());
            }
        }
    }
}
