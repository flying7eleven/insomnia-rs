use crate::annotation::FileAnnotator;
use crate::InsomniaProject;
use chrono::{TimeZone, Utc};
use clap::Clap;
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use std::borrow::Borrow;
use std::fs::{read_dir, OpenOptions};
use std::io::Write;

lazy_static! {
    static ref CORRECT_FILE_NAME_REGEX: Regex =
        Regex::new(r".*(\d{4})(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})_.*\.wav").unwrap();
}

/// A subcommand for controlling testing
#[derive(Clap)]
pub struct AnnotateCommandOptions {
    /// The folder where all wave files are stored and which can be used to generate the annotations.
    #[clap(index = 1)]
    input_folder: String,

    /// The file in which the annotation labels should be stored.
    #[clap(index = 2)]
    output_file: String,

    /// Use range information in the label text instead of just the start time of the label.
    #[clap(long)]
    range: bool,

    /// Add markers every 10 minutes (if the range is longer then that).
    #[clap(long)]
    add_sub_markers: bool,
}

pub fn run_command_annotate(options: AnnotateCommandOptions, _: InsomniaProject) {
    /*
    // ensure ta input folder was specified
    if !argument_matches.is_present("input_folder") {
        error!("No input folder specified. Cannot process files for annotation label generation.");
        return;
    }

    // ensure and output file was specified
    if !argument_matches.is_present("output_file") {
        error!("No output file for the labels specified. Cannot process files for annotation label generation.");
        return;
    }*/

    //
    let mut label_file = match OpenOptions::new()
        .append(true)
        .create(true)
        .open(options.output_file)
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

    // loop through all found files and try to process them
    let mut ordered_file_list: Vec<String> = vec![];
    for maybe_audio_file_path in read_dir(options.input_folder).unwrap() {
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
                options.add_sub_markers,
                options.range,
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
