use cpal::BuildStreamError;
use cpal::DefaultStreamConfigError;
use cpal::DeviceNameError;
use cpal::DevicesError;
use cpal::PlayStreamError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("CPAL devices error: {0}")]
    CpalDevices(#[from] DevicesError),

    #[error("CPAL stream config error: {0}")]
    CpalConfig(#[from] DefaultStreamConfigError),

    #[error("CPAL stream build error: {0}")]
    CpalStream(#[from] BuildStreamError),

    #[error("CPAL stream play error: {0}")]
    CpalPlay(#[from] PlayStreamError),

    #[error("CPAL device name error: {0}")]
    CpalDeviceName(#[from] DeviceNameError),

    #[error("CPAL device error: {0}")]
    Device(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("No device found")]
    NoDevice,

    #[error("No VB-Cable device found")]
    NoVBCableDevice,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),
}
