use clap::ArgMatches;
use insomnia::annotation::WaveMetaReader;
use lazy_static::lazy_static;
use log::{debug, error, info};
use regex::Regex;
use std::fs::{read_dir, OpenOptions};
use std::io::Write;

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

            let label_line = if no_date {
                format!(
                    "{:.2}\t{:.2}\t{}:{}:{}\n",
                    start_label, end_label, &cap[4], &cap[5], &cap[6]
                )
            } else {
                format!(
                    "{:.2}\t{:.2}\t{}.{}.{} {}:{}:{}\n",
                    start_label, end_label, &cap[3], &cap[2], &cap[1], &cap[4], &cap[5], &cap[6]
                )
            };

            write!(&mut label_file, "{}", label_line);

            start_label += duration_in_seconds;
        }
    }
}
