use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("CPAL error: {0}")]
    Cpal(#[from] cpal::StreamError),
    
    #[error("No device found")]
    NoDevice,
    
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    
    #[error("No VB-Cable device found")]
    NoVBCableDevice,
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
}
