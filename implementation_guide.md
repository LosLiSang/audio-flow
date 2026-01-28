# Rust WASAPI 音频混音器 - 实现指南与 Troubleshooting

## Part 1: 开发环境设置

### 1.1 前置要求

- **Windows 10/11** (WASAPI 需要)
- **Rust 1.70+** (`rustup update`)
- **Visual Studio C++ 构建工具**（某些依赖需要编译 C++ 代码）
- **VB-Cable 虚拟设备**（已安装）

### 1.2 项目初始化

```bash
# 创建新项目
cargo new audio-mixer --bin
cd audio-mixer

# 编辑 Cargo.toml，添加依赖
```

### 1.3 Cargo.toml 最小配置

```toml
[package]
name = "audio-mixer"
version = "0.1.0"
edition = "2021"

[dependencies]
# 核心音频库
cpal = "0.19"              # 跨平台音频 I/O

# 并发和性能
crossbeam = "0.8"          # lock-free 队列
parking_lot = "0.12"       # 快速 mutex
rayon = "1.8"              # 数据并行处理

# 采样率转换（可选，高质量）
rubato = "0.14"            # 专业级重采样

# 日志和诊断
tracing = "0.1"
tracing-subscriber = "0.3"

# CLI 界面（可选）
clap = "4.4"
anyhow = "1.0"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

---

## Part 2: 核心实现步骤

### Step 1: 设备枚举（第 1-2 天）

```rust
use cpal::traits::{DeviceTrait, HostTrait};

fn list_audio_devices() {
    let host = cpal::default_host();
    
    println!("Input Devices:");
    for (idx, device) in host.input_devices().unwrap().enumerate() {
        println!("  [{}] {}", idx, device.name().unwrap_or_default());
        
        // 显示配置支持
        if let Ok(configs) = device.supported_input_configs() {
            for config in configs {
                println!("    - Ch: {}, Rate: {} Hz", 
                    config.channels(),
                    config.min_sample_rate().0);
            }
        }
    }
    
    println!("\nOutput Devices:");
    for (idx, device) in host.output_devices().unwrap().enumerate() {
        println!("  [{}] {}", idx, device.name().unwrap_or_default());
    }
}
```

**要点**:
- 识别 "VB-Cable" 作为输出设备
- 确认支持的采样率（通常 44.1kHz、48kHz）
- 确认支持的通道数（单声道、立体声）

---

### Step 2: 创建单输入流（第 3-4 天）

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam::queue::SegQueue;
use std::sync::Arc;

fn create_single_input_stream() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host.default_input_device()
        .ok_or("No input device found")?;
    
    let config = device.default_input_config()?.config();
    println!("Input config: {} channels, {} Hz", 
        config.channels, config.sample_rate.0);
    
    // 创建队列用于跨线程传递音频数据
    let queue = Arc::new(SegQueue::new());
    let queue_clone = Arc::clone(&queue);
    
    let stream = device.build_input_stream(
        &config,
        move |data: &cpal::Data, _: &cpal::InputCallbackInfo| {
            // 数据回调：在这里读取输入音频
            if let cpal::Data::F32(buffer) = data {
                // 将音频数据推入队列
                let audio: Vec<f32> = buffer.to_vec();
                queue_clone.push(audio);
            }
        },
        |err| eprintln!("Input stream error: {}", err),
        None,
    )?;
    
    stream.play()?;
    
    // 等待流运行
    std::thread::sleep(std::time::Duration::from_secs(5));
    
    Ok(())
}
```

**关键检查点**:
- ✓ 流成功创建且没有错误
- ✓ 回调函数定期被调用
- ✓ 音频数据能从队列中读取

---

### Step 3: 创建输出流（第 5-6 天）

```rust
fn create_output_stream(
    queue: Arc<SegQueue<Vec<f32>>>,
) -> Result<cpal::Stream, Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host.default_output_device()
        .ok_or("No output device found")?;
    
    // 重要：确认这是 VB-Cable！
    let device_name = device.name()?;
    println!("Output device: {}", device_name);
    
    if !device_name.contains("Cable") {
        eprintln!("⚠️  Warning: output device is not VB-Cable!");
    }
    
    let config = device.default_output_config()?.config();
    
    let stream = device.build_output_stream(
        &config,
        move |output: &mut cpal::Data, _: &cpal::OutputCallbackInfo| {
            if let cpal::Data::F32(buffer) = output {
                if let Some(audio) = queue.pop() {
                    // 将队列中的音频复制到输出缓冲区
                    for (i, &sample) in audio.iter().enumerate() {
                        if i < buffer.len() {
                            // 防护：硬截断防止失真
                            buffer[i] = sample.clamp(-1.0, 1.0);
                        }
                    }
                } else {
                    // 队列空：输出静音
                    for sample in buffer.iter_mut() {
                        *sample = 0.0;
                    }
                }
            }
        },
        |err| eprintln!("Output stream error: {}", err),
        None,
    )?;
    
    stream.play()?;
    Ok(stream)
}
```

**关键检查点**:
- ✓ 设备确实是 VB-Cable
- ✓ 输出有声音（使用系统音量或 VB-Cable 监听）
- ✓ 音频从输入正确流向输出

---

### Step 4: 多输入混音（第 7-9 天）

```rust
struct MultiInputMixer {
    input_queues: Vec<Arc<SegQueue<Vec<f32>>>>,
    mix_buffer: Vec<f32>,
    gains: Vec<f32>,
}

impl MultiInputMixer {
    fn new(num_inputs: usize, buffer_size: usize) -> Self {
        Self {
            input_queues: vec![Arc::new(SegQueue::new()); num_inputs],
            mix_buffer: vec![0.0; buffer_size],
            gains: vec![1.0; num_inputs],
        }
    }
    
    /// 混合所有输入
    fn mix_inputs(&mut self) -> Vec<f32> {
        // 清空混合缓冲区
        self.mix_buffer.iter_mut().for_each(|s| *s = 0.0);
        
        let mut active_inputs = 0;
        
        // 从每个队列读取一块数据
        for (input_idx, queue) in self.input_queues.iter().enumerate() {
            if let Some(audio) = queue.pop() {
                active_inputs += 1;
                let gain = self.gains[input_idx];
                
                for (i, &sample) in audio.iter().enumerate() {
                    if i < self.mix_buffer.len() {
                        self.mix_buffer[i] += sample * gain;
                    }
                }
            }
        }
        
        // 归一化：除以活跃输入数量
        if active_inputs > 0 {
            let norm_factor = 1.0 / active_inputs as f32;
            for sample in &mut self.mix_buffer {
                *sample *= norm_factor;
                // 软限制防止失真
                *sample = soft_clip(*sample);
            }
        }
        
        self.mix_buffer.clone()
    }
    
    fn set_input_gain(&mut self, input_idx: usize, gain_db: f32) {
        if input_idx < self.gains.len() {
            // dB to linear: 10^(dB/20)
            self.gains[input_idx] = 10.0_f32.powf(gain_db / 20.0);
        }
    }
}

/// 软限制器：平滑处理过载
fn soft_clip(sample: f32) -> f32 {
    if sample.abs() < 0.8 {
        sample  // 低于阈值，不处理
    } else {
        sample.tanh()  // tanh 压缩曲线
    }
}
```

---

### Step 5: 处理采样率差异（第 10-11 天）

**问题**: 不同输入设备可能有不同的采样率

```rust
struct RateMismatchHandler {
    /// 缓冲区用于存储采样率不匹配的数据
    buffers: Vec<Vec<f32>>,
    /// 目标采样率
    target_rate: u32,
    /// 每个输入的当前采样率
    current_rates: Vec<u32>,
}

impl RateMismatchHandler {
    /// 检查是否需要重采样
    fn needs_resampling(&self, input_idx: usize) -> bool {
        self.current_rates.get(input_idx)
            .map(|&rate| rate != self.target_rate)
            .unwrap_or(false)
    }
    
    /// 使用 rubato 库进行高质量重采样
    fn resample_if_needed(
        &self,
        input_idx: usize,
        audio: &[f32],
    ) -> Vec<f32> {
        let input_rate = self.current_rates[input_idx];
        let output_rate = self.target_rate;
        
        if input_rate == output_rate {
            return audio.to_vec();
        }
        
        // 使用 rubato 库（需要在 Cargo.toml 中添加）
        use rubato::Resampler;
        
        let ratio = output_rate as f64 / input_rate as f64;
        let output_len = (audio.len() as f64 * ratio).ceil() as usize;
        
        // 简化：直接返回原始数据
        // 实际项目应该集成 rubato
        audio.to_vec()
    }
}
```

---

### Step 6: 时钟同步（第 12-14 天）

**问题**: 输入和输出的时钟可能不同步，导致缓冲溢出/下溢

```rust
struct AdaptiveClockManager {
    /// 缓冲区充满度（0.0 到 1.0）
    fill_ratio: f32,
    /// 重采样比例（动态调整）
    resample_ratio: f32,
    /// 低水位线（缓冲太空）
    low_water: f32,
    /// 高水位线（缓冲太满）
    high_water: f32,
}

impl AdaptiveClockManager {
    fn new() -> Self {
        Self {
            fill_ratio: 0.5,
            resample_ratio: 1.0,
            low_water: 0.3,
            high_water: 0.7,
        }
    }
    
    /// 更新重采样比例，基于缓冲充满度
    fn update(&mut self, current_fill: f32) {
        self.fill_ratio = current_fill;
        
        if current_fill > self.high_water {
            // 缓冲太满：加速输出（减速输入）
            // 方法：降低输入采样率，让输出更快消耗
            self.resample_ratio *= 0.9999;  // 减速 0.01%
        } else if current_fill < self.low_water {
            // 缓冲太空：减速输出（加速输入）
            self.resample_ratio *= 1.0001;  // 加速 0.01%
        } else {
            // 在正常范围内，逐步回到 1.0
            self.resample_ratio += (1.0 - self.resample_ratio) * 0.0001;
        }
    }
}
```

---

## Part 3: 常见问题和解决方案

### 问题 1: "No matching configuration found"

**症状**: 
```
Error: No matching configuration found
```

**原因**: 
- 设备不支持请求的采样率/通道数组合

**解决方案**:
```rust
// 不要硬编码配置，使用设备支持的配置
let config = device.default_input_config()
    .unwrap()  // 使用设备的默认配置
    .config();

// 或者枚举所有支持的配置
for config in device.supported_input_configs().unwrap() {
    println!("Supported: {} ch, {} Hz", 
        config.channels(), 
        config.sample_rate().0);
}
```

---

### 问题 2: 声音卡顿/爆音

**症状**:
- 音频定期中断
- 听到"咔咔"声

**原因**:
- 缓冲区太小
- 处理时间过长
- 内存分配导致 GC 暂停

**解决方案**:
```rust
// 1. 增加缓冲区大小
let mut output_buffer = vec![0.0; 4096];  // 从 128 增加到 4096

// 2. 预分配所有内存（不在回调中分配）
struct AudioProcessor {
    buffer: Vec<f32>,  // ✓ 在初始化时分配
}

// 3. 使用池化缓冲区重用
let buffer_pool = Arc::new(SegQueue::new());
for _ in 0..10 {
    buffer_pool.push(vec![0.0; 2048]);
}

// 4. 优化回调函数（减少计算）
// ❌ 不好：在回调中做复杂计算
device.build_input_stream(&config, |data, _| {
    expensive_dsp_processing(data);  // 太慢了！
}, ...)?

// ✓ 好：回调只做最少工作
device.build_input_stream(&config, |data, _| {
    queue.push(data.to_vec());  // 只是复制
}, ...)?

// 在单独的线程中做处理
thread::spawn(|| {
    while let Some(data) = queue.pop() {
        expensive_dsp_processing(&data);
    }
});
```

---

### 问题 3: 输出没有声音

**症状**:
- VB-Cable 没有输入信号

**排查步骤**:

```rust
fn diagnose_no_sound() {
    // 步骤 1: 确认输出设备是 VB-Cable
    let host = cpal::default_host();
    let output = host.default_output_device().unwrap();
    println!("Output device: {}", output.name().unwrap());
    // 应该显示 "VB-Cable Audio Engine"
    
    // 步骤 2: 确认流已启动
    stream.play().expect("Stream should play");
    
    // 步骤 3: 检查数据是否流经队列
    println!("Queue size: {}", queue.len());  // 应该 > 0
    
    // 步骤 4: 输出测试信号（会发出嗡嗡声）
    for sample in buffer.iter_mut() {
        *sample = 0.1;  // 常数信号，应该听到低频音
    }
    
    // 步骤 5: 检查 Windows 音量混音器
    // Win+I -> 系统 -> 声音 -> 音量混音器
    // VB-Cable 应该显示音量条在动
}
```

---

### 问题 4: 采样率不匹配导致的音频变调

**症状**:
- 声音变快或变慢
- 类似倍速播放

**原因**:
- 混音器假设所有输入都是相同采样率
- 但实际可能不同（如 44.1kHz vs 48kHz）

**解决方案**:
```rust
// 使用 rubato 库进行采样率转换
use rubato::FftFixedIn;

fn resample(
    input: &[f32],
    input_sr: u32,
    output_sr: u32,
) -> Vec<f32> {
    let ratio = output_sr as f64 / input_sr as f64;
    
    let mut resampler = FftFixedIn::<f32>::new(
        input_sr as usize,
        output_sr as usize,
        input.len(),
        1,  // 单通道
    ).unwrap();
    
    let input_frames = vec![input.to_vec()];
    let output_frames = resampler.process(&input_frames, None).unwrap();
    
    output_frames[0].clone()
}
```

---

### 问题 5: 输入音量太小或太大

**症状**:
- 输入信号幅度只有 ±0.01（太小）
- 或者经常削波（太大）

**解决方案**:
```rust
struct AutoGainControl {
    target_level: f32,  // 目标电平（-20dB = 0.1）
    current_gain: f32,
    attack_time: f32,
    release_time: f32,
}

impl AutoGainControl {
    fn process(&mut self, audio: &mut [f32]) {
        // 计算当前输入电平（RMS）
        let rms = self.calculate_rms(audio);
        
        // 计算所需增益
        let target_rms = self.target_level;
        let gain_needed = target_rms / rms.max(0.001);
        
        // 平滑增益变化（避免突突声）
        if gain_needed > self.current_gain {
            // 快速增长（attack）
            self.current_gain += (gain_needed - self.current_gain) * self.attack_time;
        } else {
            // 缓慢衰减（release）
            self.current_gain -= (self.current_gain - gain_needed) * self.release_time;
        }
        
        // 应用增益
        for sample in audio.iter_mut() {
            *sample *= self.current_gain;
        }
    }
    
    fn calculate_rms(&self, audio: &[f32]) -> f32 {
        let sum_sq: f32 = audio.iter().map(|s| s * s).sum();
        (sum_sq / audio.len() as f32).sqrt()
    }
}
```

---

### 问题 6: 内存泄漏或内存持续增长

**症状**:
- 运行 1 小时后 RAM 从 50MB 增长到 500MB

**原因**:
- 回调中不断分配内存
- 缓冲区未正确清理

**诊断**:
```bash
# 使用 cargo-valgrind 检查内存泄漏
cargo install cargo-valgrind
cargo valgrind --release
```

**解决方案**:
```rust
// ❌ 不好：每次回调分配
device.build_input_stream(&config, |data, _| {
    let mut buffer = Vec::new();  // ❌ 分配！
    buffer.extend_from_slice(data);
}, ...)?

// ✓ 好：重用缓冲区
let pool = Arc::new(SegQueue::new());
for _ in 0..10 {
    pool.push(Vec::with_capacity(2048));
}

device.build_input_stream(&config, move |data, _| {
    let mut buffer = pool.pop()
        .unwrap_or_else(|| Vec::with_capacity(2048));
    buffer.clear();
    buffer.extend_from_slice(data);
    // ... 处理 ...
    pool.push(buffer);  // 放回池
}, ...)?
```

---

## Part 4: 性能优化

### 优化 1: 线程优先级

```rust
#[cfg(target_os = "windows")]
fn set_thread_priority_high() {
    use winapi::um::processthreadsapi::SetThreadPriority;
    use winapi::um::winbase::THREAD_PRIORITY_TIME_CRITICAL;
    
    unsafe {
        SetThreadPriority(
            std::thread::current().id() as *mut _,
            THREAD_PRIORITY_TIME_CRITICAL,
        );
    }
}
```

### 优化 2: SIMD 向量化

```rust
// 使用 packed_simd 库加速混音
#[cfg(target_arch = "x86_64")]
use packed_simd::f32x4;

fn mix_simd(inputs: &[&[f32]], output: &mut [f32]) {
    for chunk in output.chunks_exact_mut(4) {
        let mut sum = f32x4::splat(0.0);
        
        for input in inputs {
            for (i, &s) in input.iter().take(4).enumerate() {
                sum.replace(i, sum.extract(i) + s);
            }
        }
        
        chunk[0] = sum.extract(0);
        chunk[1] = sum.extract(1);
        chunk[2] = sum.extract(2);
        chunk[3] = sum.extract(3);
    }
}
```

### 优化 3: 批处理

```rust
struct BatchProcessor {
    batch_size: usize,
    buffer: Vec<f32>,
}

impl BatchProcessor {
    fn process_batch(&mut self, input: &[f32]) {
        // 累积数据直到达到 batch_size
        self.buffer.extend_from_slice(input);
        
        if self.buffer.len() >= self.batch_size {
            self.process_heavy_computation(&self.buffer[..self.batch_size]);
            self.buffer.drain(0..self.batch_size);
        }
    }
    
    fn process_heavy_computation(&self, _data: &[f32]) {
        // 只在有足够数据时进行复杂处理
    }
}
```

---

## Part 5: 测试和验证

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixer_output_range() {
        let inputs = vec![
            vec![1.0; 100],  // 全量
            vec![1.0; 100],  // 全量
        ];
        
        let mut mixer = MultiInputMixer::new(2, 100);
        let output = mixer.mix(&inputs);
        
        // 验证输出在有效范围内
        assert!(output.iter().all(|&s| s >= -1.0 && s <= 1.0));
    }

    #[test]
    fn test_gain_application() {
        let mut mixer = MultiInputMixer::new(1, 10);
        mixer.set_input_gain(0, 6.0);  // +6dB = 2x
        
        let input = vec![0.5; 10];
        let output = mixer.mix(&vec![input]);
        
        // 0.5 * 2 = 1.0（达到最大）
        assert!(output.iter().all(|&s| s.abs() <= 1.0));
    }
}
```

### 集成测试

```bash
# 创建 loopback 测试：播放已知信号并录制
# 分析延迟和音质
cargo test --release -- --nocapture
```

---

## 最终清单

- [ ] 项目环境搭建
- [ ] 设备枚举工作
- [ ] 单输入流正常
- [ ] 输出到 VB-Cable 成功
- [ ] 多输入混音无失真
- [ ] 采样率转换正确
- [ ] 时钟同步稳定
- [ ] 性能检查（CPU < 10%）
- [ ] 内存检查（无泄漏）
- [ ] 长时间运行测试（> 1 小时）
- [ ] 设备热插拔处理
- [ ] UI/CLI 完成

