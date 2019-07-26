use core::fmt;
use std::error;

#[derive(Debug, Clone)]
pub struct AudioDeviceError;

impl fmt::Display for AudioDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown audio device error")
    }
}

impl error::Error for AudioDeviceError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}