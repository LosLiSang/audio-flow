// Cargo.toml - 依赖配置
/*
[package]
name = "audio-mixer"
version = "0.1.0"
edition = "2021"

[dependencies]
cpal = "0.19"
crossbeam = "0.8"
parking_lot = "0.12"
tracing = "0.1"
tracing-subscriber = "0.3"

[profile.release]
opt-level = 3
lto = true
*/

// ============================================================================
// main.rs - 使用 CPAL 的完整实现
// ============================================================================

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, StreamConfig};
use crossbeam::queue::SegQueue;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// 音频混音引擎 - CPAL 版本
struct AudioMixerCPAL {
    /// 输入流控制
    input_streams: Vec<cpal::Stream>,
    /// 输出流
    output_stream: Option<cpal::Stream>,
    /// 音频数据队列
    audio_queue: Arc<SegQueue<Vec<f32>>>,
    /// 运行状态
    running: Arc<AtomicBool>,
    /// 缓冲池，避免频繁分配
    buffer_pool: Arc<SegQueue<Vec<f32>>>,
}

impl AudioMixerCPAL {
    fn new() -> Self {
        Self {
            input_streams: Vec::new(),
            output_stream: None,
            audio_queue: Arc::new(SegQueue::new()),
            running: Arc::new(AtomicBool::new(false)),
            buffer_pool: Arc::new(SegQueue::new()),
        }
    }

    /// 初始化并列出所有音频设备
    fn list_devices() -> Result<(), Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        
        println!("Available input devices:");
        for (idx, device) in host.input_devices()?.enumerate() {
            println!("  [{}] {}", idx, device.name()?);
            
            // 显示该设备支持的配置
            if let Ok(configs) = device.supported_input_configs() {
                for config in configs {
                    println!("      - {} channels, {} Hz - {} Hz",
                        config.channels(),
                        config.min_sample_rate().0,
                        config.max_sample_rate().0);
                }
            }
        }

        println!("\nAvailable output devices:");
        for (idx, device) in host.output_devices()?.enumerate() {
            println!("  [{}] {}", idx, device.name()?);
            
            if let Ok(configs) = device.supported_output_configs() {
                for config in configs {
                    println!("      - {} channels, {} Hz - {} Hz",
                        config.channels(),
                        config.min_sample_rate().0,
                        config.max_sample_rate().0);
                }
            }
        }

        Ok(())
    }

    /// 创建输入流
    fn create_input_stream(
        &mut self,
        device: &Device,
        config: &StreamConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let queue = Arc::clone(&self.audio_queue);
        let pool = Arc::clone(&self.buffer_pool);

        let stream = device.build_input_stream(
            config,
            move |data: &cpal::Data, _: &cpal::InputCallbackInfo| {
                // 从浮点格式读取音频
                if let cpal::Data::F32(buffer) = data {
                    let mut audio_buffer = pool.pop()
                        .unwrap_or_else(|| Vec::with_capacity(buffer.len()));
                    
                    audio_buffer.clear();
                    audio_buffer.extend_from_slice(buffer);
                    
                    queue.push(audio_buffer);
                }
            },
            |err| {
                eprintln!("Input stream error: {}", err);
            },
            None,  // 不使用回调超时
        )?;

        stream.play()?;
        self.input_streams.push(stream);

        Ok(())
    }

    /// 创建输出流（写入 VB-Cable 或其他虚拟设备）
    fn create_output_stream(
        &mut self,
        device: &Device,
        config: &StreamConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let queue = Arc::clone(&self.audio_queue);
        let running = Arc::clone(&self.running);
        let pool = Arc::clone(&self.buffer_pool);

        let stream = device.build_output_stream(
            config,
            move |output: &mut cpal::Data, _: &cpal::OutputCallbackInfo| {
                if !running.load(Ordering::SeqCst) {
                    return;
                }

                if let cpal::Data::F32(buffer) = output {
                    // 简单的混音逻辑：直接从队列读取数据
                    if let Some(audio) = queue.pop() {
                        for (i, &sample) in audio.iter().enumerate() {
                            if i < buffer.len() {
                                buffer[i] = sample.min(1.0).max(-1.0);  // 硬截断
                            }
                        }
                        
                        // 放回缓冲池供重用
                        pool.push(audio);
                    } else {
                        // 没有数据：输出静音
                        for sample in buffer.iter_mut() {
                            *sample = 0.0;
                        }
                    }
                }
            },
            |err| {
                eprintln!("Output stream error: {}", err);
            },
            None,
        )?;

        stream.play()?;
        self.output_stream = Some(stream);

        Ok(())
    }

    /// 启动混音引擎
    fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        println!("Audio mixer started");
    }

    /// 停止混音引擎
    fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        println!("Audio mixer stopped");
    }
}

/// 高级示例：多输入混音器
struct MultiInputMixer {
    inputs: Vec<AudioInput>,
    mixer: SimpleAudioMixer,
    output_device: Option<Device>,
    running: Arc<AtomicBool>,
}

struct AudioInput {
    name: String,
    gain_db: f32,
    queue: Arc<SegQueue<Vec<f32>>>,
}

struct SimpleAudioMixer {
    /// 混合缓冲区
    mix_buffer: Vec<f32>,
    /// 输入增益
    gains: Vec<f32>,
}

impl SimpleAudioMixer {
    fn new(buffer_size: usize, num_inputs: usize) -> Self {
        Self {
            mix_buffer: vec![0.0; buffer_size],
            gains: vec![1.0; num_inputs],
        }
    }

    /// 混合所有输入
    fn mix(&mut self, inputs: &[Vec<f32>]) -> &[f32] {
        // 清空混合缓冲区
        self.mix_buffer.iter_mut().for_each(|s| *s = 0.0);

        // 混合所有输入
        for (input_idx, input) in inputs.iter().enumerate() {
            let gain = self.gains.get(input_idx).copied().unwrap_or(1.0);
            
            for (sample_idx, &sample) in input.iter().enumerate() {
                if sample_idx < self.mix_buffer.len() {
                    self.mix_buffer[sample_idx] += sample * gain;
                }
            }
        }

        // 归一化防止失真
        let num_inputs = inputs.len().max(1) as f32;
        for sample in &mut self.mix_buffer {
            *sample = (*sample / num_inputs).clamp(-1.0, 1.0);
        }

        &self.mix_buffer
    }

    fn set_gain(&mut self, input_idx: usize, gain: f32) {
        if input_idx < self.gains.len() {
            self.gains[input_idx] = gain;
        }
    }
}

// ============================================================================
// 完整的使用示例
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("═══════════════════════════════════════════════════════════");
    println!("    Rust WASAPI Audio Mixer - CPAL Edition");
    println!("═══════════════════════════════════════════════════════════\n");

    // 列出所有可用设备
    println!("[Step 1] Scanning audio devices...\n");
    AudioMixerCPAL::list_devices()?;

    // 创建混音器
    println!("\n[Step 2] Initializing mixer...\n");
    let mut mixer = AudioMixerCPAL::new();

    // 获取默认主机
    let host = cpal::default_host();
    println!("Host: {}\n", host.id().name());

    // 获取默认输入设备
    let input_device = host.default_input_device()
        .ok_or("No input device found")?;
    println!("Input device: {}", input_device.name()?);

    // 获取默认输出设备（应该是 VB-Cable）
    let output_device = host.default_output_device()
        .ok_or("No output device found")?;
    println!("Output device: {}\n", output_device.name()?);

    // 选择配置
    let config = input_device.default_input_config()?
        .config();
    
    println!("Configuration:");
    println!("  Channels: {}", config.channels);
    println!("  Sample rate: {} Hz", config.sample_rate.0);
    println!("  Buffer size: {} frames\n", config.buffer_size);

    // 创建输入流
    println!("[Step 3] Creating input stream...");
    mixer.create_input_stream(&input_device, &config)?;
    println!("✓ Input stream created\n");

    // 创建输出流
    println!("[Step 4] Creating output stream...");
    mixer.create_output_stream(&output_device, &config)?;
    println!("✓ Output stream created\n");

    // 启动混音
    println!("[Step 5] Starting mixer...");
    mixer.start();
    println!("✓ Mixer is running\n");

    println!("═══════════════════════════════════════════════════════════");
    println!("Mixer is active! Audio from input device is being mixed");
    println!("and sent to the output device (VB-Cable).");
    println!("");
    println!("Press Ctrl+C to stop...");
    println!("═══════════════════════════════════════════════════════════\n");

    // 运行一段时间
    thread::sleep(Duration::from_secs(60));

    // 停止
    mixer.stop();
    println!("\nMixer stopped");

    Ok(())
}

// ============================================================================
// 进阶配置示例
// ============================================================================

#[allow(dead_code)]
fn advanced_example() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    
    // 手动选择设备（而不是使用默认设备）
    let input_devices: Vec<_> = host.input_devices()?.collect();
    let output_devices: Vec<_> = host.output_devices()?.collect();

    println!("Choose input device (0-{}):", input_devices.len() - 1);
    // 假设用户选择了第一个设备
    let input_device = input_devices.into_iter().next()
        .ok_or("No input device")?;

    println!("Choose output device (0-{}):", output_devices.len() - 1);
    // 假设用户选择了设备 1（VB-Cable）
    let output_device = output_devices.into_iter().nth(1)
        .ok_or("No output device")?;

    println!("Input: {}", input_device.name()?);
    println!("Output: {}", output_device.name()?);

    // 选择特定的采样率配置
    let supported_configs = input_device.supported_input_configs()?;
    let config = supported_configs
        .filter(|c| c.channels() == 2)  // 立体声
        .filter(|c| c.sample_rate() == cpal::SampleRate(48000))  // 48kHz
        .next()
        .ok_or("No matching config")?
        .config();

    println!("Using config: {} channels, {} Hz", config.channels, config.sample_rate.0);

    Ok(())
}

// ============================================================================
// 性能监测示例
// ============================================================================

#[allow(dead_code)]
struct PerformanceMonitor {
    /// 处理时间（毫秒）
    process_time_ms: Vec<f32>,
    /// CPU 使用率（百分比）
    cpu_usage: f32,
}

impl PerformanceMonitor {
    fn new() -> Self {
        Self {
            process_time_ms: Vec::with_capacity(1000),
            cpu_usage: 0.0,
        }
    }

    /// 记录处理时间
    fn record_process_time(&mut self, elapsed_ms: f32) {
        self.process_time_ms.push(elapsed_ms);
        
        // 保持最近 1000 个样本
        if self.process_time_ms.len() > 1000 {
            self.process_time_ms.remove(0);
        }
    }

    /// 计算平均处理时间
    fn average_process_time(&self) -> f32 {
        if self.process_time_ms.is_empty() {
            0.0
        } else {
            let sum: f32 = self.process_time_ms.iter().sum();
            sum / self.process_time_ms.len() as f32
        }
    }

    /// 计算最大处理时间（峰值）
    fn max_process_time(&self) -> f32 {
        self.process_time_ms
            .iter()
            .copied()
            .fold(0.0, f32::max)
    }

    /// 打印性能统计
    fn print_stats(&self) {
        println!("Performance Statistics:");
        println!("  Avg process time: {:.2} ms", self.average_process_time());
        println!("  Max process time: {:.2} ms", self.max_process_time());
        println!("  Buffer size: {}", self.process_time_ms.len());
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_mixer() {
        let mut mixer = SimpleAudioMixer::new(100, 2);
        
        let input1 = vec![0.5; 100];
        let input2 = vec![0.3; 100];
        
        let output = mixer.mix(&[input1, input2]);
        
        // 混合后的值应该在 0-1 之间
        assert!(output.iter().all(|&s| s >= -1.0 && s <= 1.0));
        // 混合后的值应该接近 (0.5 + 0.3) / 2 = 0.4
        assert!((output[0] - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        
        monitor.record_process_time(10.5);
        monitor.record_process_time(12.3);
        monitor.record_process_time(11.8);
        
        assert!((monitor.average_process_time() - 11.53).abs() < 0.1);
        assert!(monitor.max_process_time() > 12.0);
    }
}
