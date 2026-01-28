use super::{device::DeviceManager, error::AudioError, mixer::AudioMixer};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam::queue::SegQueue;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Route {
    pub input_device_id: String,
    pub output_device_id: String,
    pub gain_db: f32,
    pub enabled: bool,
}

pub struct AudioEngine {
    pub device_manager: DeviceManager,
    pub input_streams: HashMap<String, cpal::Stream>,
    pub output_streams: HashMap<String, cpal::Stream>,
    pub routes: Vec<Route>,
    pub device_gains: HashMap<String, f32>,
    pub peak_levels: HashMap<String, Arc<AtomicU32>>,

    running: Arc<AtomicBool>,
    mixer: Arc<parking_lot::Mutex<AudioMixer>>,

    // 缓冲池
    buffer_pool: Arc<SegQueue<Vec<f32>>>,

    // 音频数据队列
    input_queues: HashMap<String, Arc<SegQueue<Vec<f32>>>>,
}

impl AudioEngine {
    pub fn new() -> Self {
        let buffer_pool = Arc::new(SegQueue::new());

        for _ in 0..10 {
            buffer_pool.push(Vec::with_capacity(512));
        }

        Self {
            device_manager: DeviceManager::new(),
            input_streams: HashMap::new(),
            output_streams: HashMap::new(),
            routes: Vec::new(),
            device_gains: HashMap::new(),
            peak_levels: HashMap::new(),
            running: Arc::new(AtomicBool::new(false)),
            mixer: Arc::new(parking_lot::Mutex::new(AudioMixer::new())),
            buffer_pool,
            input_queues: HashMap::new(),
        }
    }

    pub fn start(&mut self) -> Result<(), AudioError> {
        self.running.store(true, Ordering::SeqCst);

        let host = cpal::default_host();

        let mut input_device_ids: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut output_device_ids: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for route in &self.routes {
            if route.enabled {
                input_device_ids.insert(route.input_device_id.clone());
                output_device_ids.insert(route.output_device_id.clone());
            }
        }

        for device_id in input_device_ids {
            self.create_input_stream(&host, &device_id)?;
        }

        for device_id in output_device_ids {
            self.create_output_stream(&host, &device_id)?;
        }

        tracing::info!("Audio engine started");
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), AudioError> {
        self.running.store(false, Ordering::SeqCst);

        for stream in self.input_streams.values() {
            let _ = stream.pause();
        }
        for stream in self.output_streams.values() {
            let _ = stream.pause();
        }

        tracing::info!("Audio engine stopped");
        Ok(())
    }

    fn create_input_stream(
        &mut self,
        host: &cpal::Host,
        device_id: &str,
    ) -> Result<(), AudioError> {
        let device = self.find_input_device_by_id(host, device_id)?;
        let config = device.default_input_config()?;
        let stream_config = config.config();

        let queue = Arc::new(SegQueue::new());
        self.input_queues
            .insert(device_id.to_string(), Arc::clone(&queue));

        let peak_detector = Arc::new(AtomicU32::new(0));
        self.peak_levels
            .insert(device_id.to_string(), Arc::clone(&peak_detector));

        let queue_clone = Arc::clone(&queue);
        let pool_clone = Arc::clone(&self.buffer_pool);
        let peak_clone = Arc::clone(&peak_detector);
        let running_clone = Arc::clone(&self.running);
        let device_id_clone = device_id.to_string();

        let stream = device.build_input_stream(
            &stream_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if !running_clone.load(Ordering::SeqCst) {
                    return;
                }

                let peak = data.iter().fold(0.0f32, |max, &s| max.max(s.abs()));
                peak_clone.store((peak * 1000.0) as u32, Ordering::SeqCst);

                let mut audio_buffer = pool_clone
                    .pop()
                    .unwrap_or_else(|| Vec::with_capacity(data.len()));

                audio_buffer.clear();
                audio_buffer.extend_from_slice(data);

                queue_clone.push(audio_buffer);
            },
            move |err| {
                tracing::error!("Input stream error for device {}: {}", device_id_clone, err);
            },
            None,
        )?;

        stream.play()?;
        self.input_streams.insert(device_id.to_string(), stream);

        tracing::info!("Created input stream for device: {}", device_id);
        Ok(())
    }

    fn create_output_stream(
        &mut self,
        host: &cpal::Host,
        device_id: &str,
    ) -> Result<(), AudioError> {
        let device = self.find_output_device_by_id(host, device_id)?;
        let config = device.default_output_config()?;
        let stream_config = config.config();

        let routes_for_output: Vec<Route> = self
            .routes
            .iter()
            .filter(|r| r.output_device_id == device_id && r.enabled)
            .cloned()
            .collect();

        let mut input_sources: Vec<(Arc<SegQueue<Vec<f32>>>, f32)> = Vec::new();

        for route in &routes_for_output {
            if let Some(queue) = self.input_queues.get(&route.input_device_id) {
                let gain_linear = 10.0_f32.powf(route.gain_db / 20.0);
                input_sources.push((Arc::clone(queue), gain_linear));
            }
        }

        let running_clone = Arc::clone(&self.running);
        let pool_clone = Arc::clone(&self.buffer_pool);
        let device_id_clone = device_id.to_string();

        let stream = device.build_output_stream(
            &stream_config,
            move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                if !running_clone.load(Ordering::SeqCst) {
                    return;
                }

                let mut mix_buffer = vec![0.0f32; output.len()];
                let mut total_samples = 0;

                for (queue, gain) in &input_sources {
                    if let Some(audio) = queue.pop() {
                        total_samples += audio.len();

                        for (i, &sample) in audio.iter().enumerate() {
                            if i < mix_buffer.len() {
                                mix_buffer[i] += sample * gain;
                            }
                        }

                        pool_clone.push(audio);
                    }
                }

                if total_samples > 0 {
                    let num_inputs = input_sources.len().max(1) as f32;
                    for sample in &mut mix_buffer {
                        *sample = (*sample / num_inputs).clamp(-1.0, 1.0);
                    }
                }

                output.copy_from_slice(&mix_buffer);
            },
            move |err| {
                tracing::error!(
                    "Output stream error for device {}: {}",
                    device_id_clone,
                    err
                );
            },
            None,
        )?;

        stream.play()?;
        self.output_streams.insert(device_id.to_string(), stream);

        tracing::info!("Created output stream for device: {}", device_id);
        Ok(())
    }

    fn find_input_device_by_id(
        &self,
        host: &cpal::Host,
        device_id: &str,
    ) -> Result<cpal::Device, AudioError> {
        for device in host.input_devices()? {
            let name = device.name()?;
            let generated_id = format!("{}_input", name);
            if generated_id == device_id {
                return Ok(device);
            }
        }
        Err(AudioError::DeviceNotFound(device_id.to_string()))
    }

    fn find_output_device_by_id(
        &self,
        host: &cpal::Host,
        device_id: &str,
    ) -> Result<cpal::Device, AudioError> {
        for device in host.output_devices()? {
            let name = device.name()?;
            let generated_id = format!("{}_output", name);
            if generated_id == device_id {
                return Ok(device);
            }
        }
        Err(AudioError::DeviceNotFound(device_id.to_string()))
    }

    pub fn add_route(&mut self, route: Route) -> Result<(), AudioError> {
        self.routes.push(route);
        tracing::info!(
            "Added route: {} -> {}",
            route.input_device_id,
            route.output_device_id
        );
        Ok(())
    }

    pub fn remove_route(&mut self, input_id: &str, output_id: &str) -> Result<(), AudioError> {
        self.routes
            .retain(|r| !(r.input_device_id == input_id && r.output_device_id == output_id));
        tracing::info!("Removed route: {} -> {}", input_id, output_id);
        Ok(())
    }

    pub fn set_gain(&mut self, device_id: &str, gain_db: f32) -> Result<(), AudioError> {
        self.device_gains.insert(device_id.to_string(), gain_db);

        for route in &mut self.routes {
            if route.input_device_id == device_id {
                route.gain_db = gain_db;
            }
        }

        tracing::info!("Set gain for device {}: {} dB", device_id, gain_db);
        Ok(())
    }

    pub fn get_peak_levels(&self) -> HashMap<String, f32> {
        self.peak_levels
            .iter()
            .map(|(id, level)| (id.clone(), level.load(Ordering::SeqCst) as f32 / 1000.0))
            .collect()
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}
