use core::{fmt, mem};
use log::error;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufReader, Read};

#[derive(Debug)]
pub enum ReadError {
    Format(ReadErrorKind),
    Io(io::Error),
}

#[derive(Debug)]
pub enum ReadErrorKind {
    NotARiffFile,
    NotAWaveFile,
}

impl ReadErrorKind {
    fn to_string(&self) -> &str {
        match *self {
            ReadErrorKind::NotARiffFile => "not a RIFF file",
            ReadErrorKind::NotAWaveFile => "not a WAVE file",
        }
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReadError::Format(ref err_kind) => write!(f, "Format error: {}", err_kind.to_string()),
            ReadError::Io(ref err) => write!(f, "IO error: {}", err),
        }
    }
}

pub struct WaveMetaReader {
    file_size_in_byte: u64,
    bytes_per_second: u32,
}

impl WaveMetaReader {
    pub fn from_file(path: &str) -> Result<WaveMetaReader, ReadError> {
        // try to open the audio file
        let mut file_handle = match File::open(path) {
            Ok(file) => file,
            Err(e) => return Err(ReadError::Io(e)),
        };

        // read the first four bytes. they should contain the RIFF header if the file is valid
        let mut riff_header = [0; 4];
        file_handle.read(&mut riff_header);

        // if the file does not have a RIFF header, we can not process it any further
        if !vec![82, 73, 70, 70].eq(&riff_header) {
            return Err(ReadError::Format(ReadErrorKind::NotARiffFile));
        }

        // read the file size from the file based on the file header
        let mut file_size_based_on_heade_buffer = [0; 4];
        file_handle.read(&mut file_size_based_on_heade_buffer);
        let file_size_based_on_header: u32 = unsafe { mem::transmute(file_size_based_on_heade_buffer) };

        // the next four bytes should be the WAVE header
        let mut wave_header = [0; 4];
        file_handle.read(&mut wave_header);

        // if the file does not have a RIFF header, we can not process it any further
        if !vec![87, 65, 86, 69].eq(&wave_header) {
            return Err(ReadError::Format(ReadErrorKind::NotAWaveFile));
        }

        //
        Ok(WaveMetaReader {
            bytes_per_second: 0,
            file_size_in_byte: 0,
        })
    }

    pub fn get_duration(&self) -> u64 {
        0
    }
}
