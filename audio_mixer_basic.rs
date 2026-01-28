// Rust WASAPI 音频混音器 - 基础框架
// 这个示例展示核心概念，需要配合 cpal 库使用

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::Duration;

/// 音频混音器核心结构
pub struct AudioMixer {
    /// 每个输入通道的增益（dB）
    input_gains: Vec<f32>,
    /// 输出增益（dB）
    output_gain: f32,
    /// 是否启用峰值检测
    enable_peak_detection: bool,
    /// 当前检测到的峰值
    current_peak: f32,
}

impl AudioMixer {
    pub fn new() -> Self {
        Self {
            input_gains: vec![0.0],  // 默认 0dB（无增益）
            output_gain: 0.0,
            enable_peak_detection: true,
            current_peak: 0.0,
        }
    }

    /// 添加一个输入通道
    pub fn add_input(&mut self, gain_db: f32) {
        self.input_gains.push(gain_db);
    }

    /// 设置特定输入的增益
    pub fn set_input_gain(&mut self, input_index: usize, gain_db: f32) {
        if input_index < self.input_gains.len() {
            self.input_gains[input_index] = gain_db;
        }
    }

    /// 混合多个音频输入
    /// 
    /// # 参数
    /// - `inputs`: 多个输入音频数组 &[Vec<f32>]
    /// - `output`: 输出缓冲区的可变引用
    ///
    /// # 示例
    /// ```ignore
    /// let input1 = vec![0.1, 0.2, 0.3];
    /// let input2 = vec![0.2, 0.1, 0.15];
    /// let mut output = vec![0.0; 3];
    /// mixer.mix(&[&input1, &input2], &mut output);
    /// ```
    pub fn mix(&mut self, inputs: &[&[f32]], output: &mut [f32]) {
        let num_samples = output.len();
        output.iter_mut().for_each(|s| *s = 0.0);

        // 步骤 1: 相加所有输入
        for (input_idx, input) in inputs.iter().enumerate() {
            let gain_linear = self.db_to_linear(self.input_gains[input_idx]);

            for (sample_idx, &sample) in input.iter().enumerate() {
                if sample_idx < num_samples {
                    output[sample_idx] += sample * gain_linear;
                }
            }
        }

        // 步骤 2: 应用输出增益和软限制
        let output_gain_linear = self.db_to_linear(self.output_gain);
        let num_inputs = inputs.len().max(1) as f32;

        for sample in output.iter_mut() {
            *sample = *sample / num_inputs * output_gain_linear;

            // 软限制：使用 tanh 压缩曲线平滑处理过载
            // 当信号接近 ±1.0 时开始压缩
            *sample = self.soft_limiter(*sample, 0.9);

            // 峰值检测
            if self.enable_peak_detection {
                let abs_sample = sample.abs();
                if abs_sample > self.current_peak {
                    self.current_peak = abs_sample;
                } else {
                    // 峰值衰减（类似峰值表）
                    self.current_peak *= 0.999;
                }
            }
        }
    }

    /// dB 转线性增益
    /// 0 dB = 1.0 (无增益)
    /// 6 dB ≈ 2.0 (音量翻倍)
    /// -6 dB ≈ 0.5 (音量减半)
    fn db_to_linear(&self, db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }

    /// 软限制器，使用 tanh 压缩曲线
    /// threshold: 0.0-1.0，超过此值开始压缩
    fn soft_limiter(&self, sample: f32, threshold: f32) -> f32 {
        if sample.abs() < threshold {
            sample
        } else {
            // tanh 会平滑地将信号压缩到 [-1, 1] 范围
            sample.tanh()
        }
    }

    /// 获取当前峰值
    pub fn get_peak(&self) -> f32 {
        self.current_peak
    }

    /// 重置峰值检测器
    pub fn reset_peak(&mut self) {
        self.current_peak = 0.0;
    }
}

/// 环形缓冲区：用于跨线程存储音频数据
/// 这是处理不同采样率/时钟频率的关键
pub struct RingBuffer {
    buffer: Vec<f32>,
    write_pos: usize,
    read_pos: usize,
    capacity: usize,
}

impl RingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0.0; capacity],
            write_pos: 0,
            read_pos: 0,
            capacity,
        }
    }

    /// 写入样本
    pub fn write(&mut self, samples: &[f32]) -> usize {
        let mut written = 0;

        for &sample in samples {
            self.buffer[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.capacity;

            // 检查缓冲区溢出（写指针追上读指针）
            if self.write_pos == self.read_pos {
                // 缓冲满！移动读指针以覆盖最旧的数据
                self.read_pos = (self.read_pos + 1) % self.capacity;
                // 注意：这会导致音频丢失，理想情况下应该通知调用者
            }

            written += 1;
        }

        written
    }

    /// 读取样本
    pub fn read(&mut self, samples: &mut [f32]) -> usize {
        let mut read = 0;

        for sample in samples.iter_mut() {
            if self.read_pos == self.write_pos {
                // 缓冲空，插入静音
                *sample = 0.0;
            } else {
                *sample = self.buffer[self.read_pos];
                self.read_pos = (self.read_pos + 1) % self.capacity;
            }
            read += 1;
        }

        read
    }

    /// 获取缓冲区充满度（0.0 到 1.0）
    pub fn fill_ratio(&self) -> f32 {
        let filled = if self.write_pos >= self.read_pos {
            self.write_pos - self.read_pos
        } else {
            self.capacity - self.read_pos + self.write_pos
        };

        filled as f32 / self.capacity as f32
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.write_pos = 0;
        self.read_pos = 0;
    }
}

/// 采样率转换器（简化版：线性插值）
/// 实际项目应该使用 `rubato` 库获得更好的音质
pub struct SimpleResampler {
    /// 当前位置（可以是小数）
    position: f32,
    /// 采样率比例：input_rate / output_rate
    ratio: f32,
}

impl SimpleResampler {
    pub fn new(input_rate: u32, output_rate: u32) -> Self {
        Self {
            position: 0.0,
            ratio: input_rate as f32 / output_rate as f32,
        }
    }

    /// 进行简单的线性插值重采样
    /// input: 输入样本
    /// output: 输出缓冲区
    pub fn resample(&mut self, input: &[f32], output: &mut [f32]) {
        for output_sample in output.iter_mut() {
            let index = self.position as usize;

            // 检查边界
            if index >= input.len() {
                *output_sample = 0.0;
                self.position += self.ratio;
                continue;
            }

            // 线性插值
            let frac = self.position - index as f32;
            let sample1 = input[index];
            let sample2 = if index + 1 < input.len() {
                input[index + 1]
            } else {
                0.0
            };

            *output_sample = sample1 * (1.0 - frac) + sample2 * frac;

            self.position += self.ratio;
        }

        // 重置位置以处理周期性
        if self.position >= input.len() as f32 {
            self.position -= input.len() as f32;
        }
    }

    /// 重置重采样器的状态
    pub fn reset(&mut self) {
        self.position = 0.0;
    }
}

/// 输入源结构体：代表一个音频输入设备
pub struct InputSource {
    pub device_name: String,
    pub sample_rate: u32,
    pub channel_count: u32,
    pub gain_db: f32,
    pub enabled: bool,
}

/// 音频引擎：协调所有输入、混音和输出
pub struct AudioEngine {
    mixer: AudioMixer,
    input_sources: Vec<InputSource>,
    ring_buffers: Vec<RingBuffer>,
    output_buffer: Vec<f32>,
    running: Arc<AtomicBool>,
}

impl AudioEngine {
    pub fn new(output_sample_rate: u32, buffer_size: usize) -> Self {
        Self {
            mixer: AudioMixer::new(),
            input_sources: Vec::new(),
            ring_buffers: Vec::new(),
            output_buffer: vec![0.0; buffer_size],
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 添加输入源
    pub fn add_input_source(
        &mut self,
        device_name: String,
        sample_rate: u32,
        channel_count: u32,
    ) {
        self.input_sources.push(InputSource {
            device_name,
            sample_rate,
            channel_count,
            gain_db: 0.0,
            enabled: true,
        });

        // 为每个输入创建一个环形缓冲区
        self.ring_buffers.push(RingBuffer::new(sample_rate as usize * 2)); // 2秒缓冲

        self.mixer.add_input(0.0);
    }

    /// 启动音频引擎
    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
    }

    /// 停止音频引擎
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// 检查是否运行中
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

// ============================================================================
// 使用示例
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixer_basic() {
        let mut mixer = AudioMixer::new();

        // 两个输入，各含 3 个样本
        let input1 = vec![0.1, 0.2, 0.3];
        let input2 = vec![0.2, 0.1, 0.15];

        let mut output = vec![0.0; 3];

        mixer.mix(&[&input1, &input2], &mut output);

        // 简单的检查：混合后的样本应该是两个输入的平均值
        // output[0] = (0.1 + 0.2) / 2 = 0.15
        println!("Mixed output: {:?}", output);
        assert!(output[0] > 0.0 && output[0] < 1.0);
    }

    #[test]
    fn test_ring_buffer() {
        let mut buffer = RingBuffer::new(10);

        // 写入数据
        let input = vec![0.1, 0.2, 0.3];
        buffer.write(&input);

        // 读取数据
        let mut output = vec![0.0; 3];
        buffer.read(&mut output);

        assert_eq!(output, vec![0.1, 0.2, 0.3]);
        println!("Ring buffer test passed");
    }

    #[test]
    fn test_ring_buffer_fill_ratio() {
        let mut buffer = RingBuffer::new(100);

        let input = vec![0.5; 50];
        buffer.write(&input);

        let ratio = buffer.fill_ratio();
        println!("Buffer fill ratio: {:.2}%", ratio * 100.0);
        assert!(ratio > 0.4 && ratio < 0.6);
    }

    #[test]
    fn test_resampler() {
        let mut resampler = SimpleResampler::new(48000, 44100);

        // 输入数据（48kHz）
        let input = vec![0.1, 0.2, 0.3, 0.4, 0.5];

        // 输出数据（44.1kHz）
        let mut output = vec![0.0; 5];

        resampler.resample(&input, &mut output);

        println!("Resampled output: {:?}", output);
        assert!(!output.iter().any(|&s| s.is_nan()));
    }

    #[test]
    fn test_soft_limiter() {
        let mixer = AudioMixer::new();

        // 测试软限制器
        let sample = 1.5;  // 超过最大值
        let limited = mixer.soft_limiter(sample, 0.9);

        println!("Original: {}, Limited: {}", sample, limited);
        assert!(limited.abs() <= 1.0);
    }
}

fn main() {
    println!("Rust Audio Mixer Framework");
    println!("这是一个基础框架，展示核心概念");
    println!("");
    println!("关键组件:");
    println!("  1. AudioMixer - 混音核心逻辑");
    println!("  2. RingBuffer - 跨线程音频缓冲");
    println!("  3. SimpleResampler - 采样率转换");
    println!("  4. AudioEngine - 整体协调");
    println!("");
    println!("运行单元测试: cargo test -- --nocapture");
}
