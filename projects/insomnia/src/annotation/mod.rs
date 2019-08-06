use chrono::{Duration as OldDuration, NaiveDateTime};
use core::{fmt, mem};
use log::{debug, error};
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

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
        debug!("The data block for {} is {} bytes long with {} bits/sample, a sample rate of {} samples/second and {} channels, this results in {} samples and a duration of {} seconds.", Path::new(path).file_name().unwrap().to_str().unwrap(), data_block_size_in_byte, bits_per_sample, samples_per_second, channels, number_of_samples, duration);
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

pub struct AnnotationLabel {
    start_marker: f32,
    end_marker: f32,
    used_label: String,
}

impl AnnotationLabel {
    pub fn get_label_line(&self) -> String {
        format!(
            "{:.02}\t{:.02}\t{}\n",
            self.start_marker, self.end_marker, self.used_label
        )
    }
}

pub struct FileAnnotator {
    file_duration_in_seconds: u64,
    slice_duration_in_seconds: u64,
    file_start_time_in_seconds: u64,
    file_base_time: NaiveDateTime,
    max_annotations: usize,
    next_annotation_idx: usize,
    last_start_time: f32,
    is_range: bool,
}

impl FileAnnotator {
    pub fn from(
        file_name: &str,
        file_start_date: NaiveDateTime,
        start_time: u64,
        add_sub_markers: bool,
    ) -> Option<FileAnnotator> {
        // try to get the meta information from the audiof ile itself
        let maybe_meta_reader = WaveMetaReader::from_file(file_name);
        if !maybe_meta_reader.is_ok() {
            return None;
        }
        let meta_reader = maybe_meta_reader.unwrap();

        // if we should add sub markers, determine a length for a sub-marker
        let slice_length = if add_sub_markers {
            (meta_reader.get_duration() / 6.0) as u64
        } else {
            meta_reader.get_duration() as u64
        };

        // determine the nmber of labels we want to set for this part
        let max_annotations = if add_sub_markers { 6 } else { 1 };

        // create the new file annotator
        Some(FileAnnotator {
            file_duration_in_seconds: meta_reader.get_duration() as u64,
            slice_duration_in_seconds: slice_length,
            file_start_time_in_seconds: start_time,
            last_start_time: start_time as f32,
            max_annotations,
            is_range: true,
            file_base_time: file_start_date,
            next_annotation_idx: 0,
        })
    }

    pub fn get_end_time(&self) -> u64 {
        self.file_start_time_in_seconds + self.file_duration_in_seconds as u64
    }

    pub fn get_max_labels(&self) -> usize {
        self.max_annotations
    }
}

impl Iterator for FileAnnotator {
    type Item = AnnotationLabel;

    fn next(&mut self) -> Option<Self::Item> {
        // if we reached the max. number of annotations, return None to signal that
        if self.next_annotation_idx >= self.max_annotations {
            return None;
        }

        // since we return a new annotation, increase the id for the next one
        self.next_annotation_idx += 1;

        // calculate the required times for the labels
        let old_last_start_time = self.last_start_time;
        let end_marker_offset = if self.is_range {
            self.slice_duration_in_seconds as f32
        } else {
            0.0
        };
        self.last_start_time += end_marker_offset;

        let new_end_time_for_slice = self.file_base_time
            + OldDuration::seconds(
                self.slice_duration_in_seconds as i64 * self.next_annotation_idx as i64,
            );

        let actual_slice_start_time = if self.max_annotations > 1 {
            self.file_base_time
                + OldDuration::seconds(
                    self.slice_duration_in_seconds as i64 * (self.next_annotation_idx as i64 - 1),
                )
        } else {
            self.file_base_time
        };

        let used_label = if self.is_range {
            format!(
                "{} - {}",
                actual_slice_start_time.format("%H:%M:%S").to_string(),
                new_end_time_for_slice.format("%H:%M:%S").to_string()
            )
        } else {
            actual_slice_start_time
                .format("%d.%m.%Y %H:%M:%S")
                .to_string()
        };

        // return the new annotation label
        Some(AnnotationLabel {
            start_marker: old_last_start_time,
            end_marker: self.last_start_time,
            used_label,
        })
    }
}
