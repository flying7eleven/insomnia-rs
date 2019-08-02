use clap::ArgMatches;
use insomnia::annotation::WaveMetaReader;
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use std::fs::read_dir;

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

        //
        for cap in CORRECT_FILE_NAME_REGEX.captures_iter(audio_file_path) {
            println!("Year: {}", &cap[1]);
            println!("Month: {}", &cap[2]);
            println!("Day: {}", &cap[3]);
            println!("Hour: {}", &cap[4]);
            println!("Minute: {}", &cap[5]);
            println!("Second: {}", &cap[6]);

            let maybe_meta_reader = WaveMetaReader::from_file(audio_file_path);
            if maybe_meta_reader.is_err() {
                error!("Could not read the meta information of the file.");
                continue;
            }
            let meta_reader = maybe_meta_reader.unwrap();
            let duration_in_seconds = meta_reader.get_duration();

            println!("Duration: {}s", duration_in_seconds);
            println!("------")
        }
    }
}
