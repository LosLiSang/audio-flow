use cpal::BuildStreamError;
use cpal::DefaultStreamConfigError;
use cpal::DeviceNameError;
use cpal::DevicesError;
use cpal::PlayStreamError;
use std::io::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("CPAL error: {0}")]
    Cpal(#[from] DevicesError),

    #[error("CPAL error: {0}")]
    Cpal(#[from] DefaultStreamConfigError),

    #[error("CPAL error: {0}")]
    Cpal(#[from] BuildStreamError),

    #[error("CPAL error: {0}")]
    Cpal(#[from] PlayStreamError),

    #[error("CPAL error: {0}")]
    Cpal(#[from] DeviceNameError),

    #[error("CPAL error: {0}")]
    Cpal(#[from] DefaultStreamConfigError),

    #[error("No device found")]
    NoDevice,

    #[error("No VB-Cable device found")]
    NoVBCableDevice,

    #[error("IO error: {0}")]
    Io(#[from] Error),

    #[error("Configuration error: {0}")]
    Config(String),
}

impl From<BuildStreamError> for AudioError {
    fn from(err: BuildStreamError) -> Self {
        AudioError::Cpal(err.to_string())
    }
}

impl From<DefaultStreamConfigError> for AudioError {
    fn from(err: DefaultStreamConfigError) -> Self {
        AudioError::Cpal(err.to_string())
    }
}

impl From<DeviceNameError> for AudioError {
    fn from(err: DeviceNameError) -> Self {
        AudioError::Cpal(err.to_string())
    }
}

impl From<DevicesError> for AudioError {
    fn from(err: DevicesError) -> Self {
        AudioError::Cpal(err.to_string())
    }
}

impl From<PlayStreamError> for AudioError {
    fn from(err: PlayStreamError) -> Self {
        AudioError::Cpal(err.to_string())
    }
}

impl From<std::io::Error> for AudioError {
    fn from(err: Error) -> Self {
        AudioError::Io(err.to_string())
    }
}
