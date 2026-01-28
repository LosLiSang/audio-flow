use cpal::traits::{DeviceTrait, HostTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub is_input: bool,
    pub is_output: bool,
    pub sample_rate: u32,
    pub channels: u16,
    pub is_vb_cable: bool,
}

pub struct DeviceManager {
    host: cpal::Host,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    pub fn list_devices(&self) -> Result<Vec<DeviceInfo>, super::AudioError> {
        let mut devices = Vec::new();

        for device in self.host.input_devices()? {
            let name = device.description()?.to_string();
            let config = device.default_input_config()?;

            devices.push(DeviceInfo {
                id: self.generate_device_id(&name, true),
                name: name.clone(),
                is_input: true,
                is_output: false,
                sample_rate: config.sample_rate(),
                channels: config.channels(),
                is_vb_cable: self.is_vb_cable(&name),
            });
        }

        for device in self.host.output_devices()? {
            let name = device.description()?.to_string();
            let config = device.default_output_config()?;

            devices.push(DeviceInfo {
                id: self.generate_device_id(&name, false),
                name: name.clone(),
                is_input: false,
                is_output: true,
                sample_rate: config.sample_rate(),
                channels: config.channels(),
                is_vb_cable: self.is_vb_cable(&name),
            });
        }

        Ok(devices)
    }

    fn generate_device_id(&self, name: &str, is_input: bool) -> String {
        format!("{}_{}", name, if is_input { "input" } else { "output" })
    }

    fn is_vb_cable(&self, name: &str) -> bool {
        name.contains("Cable") || name.contains("VB Audio Cable")
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}
