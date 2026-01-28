use cpal::BuildStreamError;
use cpal::DefaultStreamConfigError;
use cpal::DefaultStreamConfigError;
use cpal::DeviceNameError;
use cpal::DevicesError;
use cpal::PlayStreamError;
use std::io::Error;
use thiserror::Error;

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
