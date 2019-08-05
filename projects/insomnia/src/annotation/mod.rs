use core::{fmt, mem};
use log::{debug, error};
use std::fs::File;
use std::io;
use std::io::Read;

#[derive(Debug)]
pub enum ReadError {
    Format(ReadErrorKind),
    Io(io::Error),
}

#[derive(Debug)]
pub enum ReadErrorKind {
    NotARiffFile,
    NotAWaveFile,
    NoFormatChunk,
    NoDataChunk,
}

impl ReadErrorKind {
    fn to_string(&self) -> &str {
        match *self {
            ReadErrorKind::NotARiffFile => "not a RIFF file",
            ReadErrorKind::NotAWaveFile => "not a WAVE file",
            ReadErrorKind::NoFormatChunk => "no format chunk found",
            ReadErrorKind::NoDataChunk => "no data chunk found",
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
    _data_block_size_in_byte: u32,
    _bits_per_sample: u16,
    _channels: u16,
    _samples_per_second: u32,
    duration_in_seconds: f64,
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
        let _ = file_handle.read(&mut riff_header);

        // if the file does not have a RIFF header, we can not process it any further
        if !vec![82, 73, 70, 70].eq(&riff_header) {
            return Err(ReadError::Format(ReadErrorKind::NotARiffFile));
        }

        // read the file size from the file based on the file header
        let mut file_size_based_on_heade_buffer = [0; 4];
        let _ = file_handle.read(&mut file_size_based_on_heade_buffer);

        // the next four bytes should be the WAVE header
        let mut wave_header = [0; 4];
        let _ = file_handle.read(&mut wave_header);

        // if the file does not have a RIFF header, we can not process it any further
        if !vec![87, 65, 86, 69].eq(&wave_header) {
            return Err(ReadError::Format(ReadErrorKind::NotAWaveFile));
        }

        // the next three four should be 'fmt '
        let mut fmt_header = [0; 4];
        let _ = file_handle.read(&mut fmt_header);

        // ensure we read the 'fmt' we expected
        if !vec![102, 109, 116, 32].eq(&fmt_header) {
            return Err(ReadError::Format(ReadErrorKind::NoFormatChunk));
        }

        // skip the next few bytes (not interested in them for our use)
        let mut _unused = [0; 6];
        let _ = file_handle.read(&mut _unused);

        //
        let mut channels_byte = [0; 2];
        let _ = file_handle.read(&mut channels_byte);
        let channels: u16 = unsafe { mem::transmute(channels_byte) };

        //
        let mut samples_per_second_byte = [0; 4];
        let _ = file_handle.read(&mut samples_per_second_byte);
        let samples_per_second: u32 = unsafe { mem::transmute(samples_per_second_byte) };

        // skip the next few bytes (not interested in them for our use)
        let mut _unused2 = [0; 6];
        let _ = file_handle.read(&mut _unused2);

        //
        let mut bits_per_sample_buffer = [0; 2];
        let _ = file_handle.read(&mut bits_per_sample_buffer);
        let bits_per_sample: u16 = unsafe { mem::transmute(bits_per_sample_buffer) };

        // the next three four should be 'data'
        let mut data_header = [0; 4];
        let _ = file_handle.read(&mut data_header);

        // ensure we read the 'fmt' we expected
        if !vec![100, 97, 116, 97].eq(&data_header) {
            error!("{:?}", data_header);
            return Err(ReadError::Format(ReadErrorKind::NoDataChunk));
        }

        // get the size of the data block in byte
        let mut data_block_size_in_byte_buffer = [0; 4];
        let _ = file_handle.read(&mut data_block_size_in_byte_buffer);
        let data_block_size_in_byte: u32 =
            unsafe { mem::transmute(data_block_size_in_byte_buffer) };

        // calculate the information required for further processing
        let number_of_samples =
            data_block_size_in_byte / u32::from(bits_per_sample / 8) / u32::from(channels);
        let duration = f64::from(number_of_samples) / f64::from(samples_per_second);

        // return the gathered information
        debug!("The data block is {} bytes long with {} bits/sample, a sample rate of {} samples/second and {} channels, this results in {} samples and a duration of {} seconds.", data_block_size_in_byte, bits_per_sample, samples_per_second, channels, number_of_samples, duration);
        Ok(WaveMetaReader {
            _data_block_size_in_byte: data_block_size_in_byte,
            _bits_per_sample: bits_per_sample,
            _channels: channels,
            _samples_per_second: samples_per_second,
            duration_in_seconds: duration,
        })
    }

    pub fn get_duration(&self) -> f64 {
        self.duration_in_seconds
    }
}
