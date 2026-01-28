# AGENTS.md

This repository contains a Rust WASAPI audio mixer implementation. This guide helps AI agents work effectively in this codebase.

## Build, Test, and Lint Commands

```bash
# Build the project
cargo build

# Build release version with optimizations
cargo build --release

# Run all tests
cargo test

# Run a single test with output
cargo test test_mixer_basic -- --nocapture

# Run tests for a specific module
cargo test mixer::tests

# Run all tests with output
cargo test -- --nocapture

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Run clippy with more strict checks
cargo clippy --all-targets --all-features -- -D warnings

# Build documentation
cargo doc --open
```

## Project Structure

- `audio_mixer_basic.rs` - Core mixer framework with AudioMixer, RingBuffer, SimpleResampler, and AudioEngine
- `audio_mixer_cpal.rs` - Full CPAL implementation for real-time audio I/O
- `audio_mixer_guide.md` - Technical guide covering architecture and key challenges
- `implementation_guide.md` - Detailed implementation steps and troubleshooting

## Code Style Guidelines

### Imports

```rust
// std imports first
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

// external crates second
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam::queue::SegQueue;

// local imports last
use crate::mixer::AudioMixer;
```

### Formatting

- Use 4-space indentation
- Maximum line width: 100 characters
- Use `cargo fmt` for automatic formatting
- No trailing whitespace

### Types

```rust
// Prefer f32 for audio samples (32-bit float)
pub struct AudioMixer {
    input_gains: Vec<f32>,
    output_gain: f32,
    current_peak: f32,
}

// Use Vec<f32> for audio buffers, avoid allocations in callbacks
pub struct RingBuffer {
    buffer: Vec<f32>,
    write_pos: usize,
    read_pos: usize,
}
```

### Naming Conventions

- Struct names: PascalCase (`AudioMixer`, `RingBuffer`)
- Function names: snake_case (`add_input`, `set_gain`)
- Constants: SCREAMING_SNAKE_CASE (`MAX_BUFFER_SIZE`)
- Private fields: snake_case (`input_gains`, `current_peak`)
- Boolean variables: `is_` or `has_` prefix (`is_running`, `has_data`)

### Error Handling

```rust
// Use Result<T, Box<dyn std::error::Error>> for functions that can fail
pub fn create_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    let device = host.default_input_device()
        .ok_or("No input device found")?;
    Ok(())
}

// Use unwrap_or_else for recoverable defaults
let mut buffer = pool.pop()
    .unwrap_or_else(|| Vec::with_capacity(2048));

// Use eprintln! for errors in callbacks
|err| eprintln!("Input stream error: {}", err)
```

### Documentation

```rust
/// Audio mixer core structure
///
/// Mixes multiple audio inputs with individual gain control.
pub struct AudioMixer {
    /// Gain for each input channel in dB
    input_gains: Vec<f32>,
    /// Output gain in dB
    output_gain: f32,
}

/// Mix multiple audio inputs
///
/// # Arguments
///
/// * `inputs` - Multiple input audio arrays
/// * `output` - Mutable reference to output buffer
///
/// # Example
/// ```ignore
/// let input1 = vec![0.1, 0.2, 0.3];
/// let input2 = vec![0.2, 0.1, 0.15];
/// let mut output = vec![0.0; 3];
/// mixer.mix(&[&input1, &input2], &mut output);
/// ```
pub fn mix(&mut self, inputs: &[&[f32]], output: &mut [f32]) {
    // implementation
}
```

### Performance Guidelines

**CRITICAL**: Audio callbacks must be real-time and avoid:
- Memory allocation (no `Vec::new()`, `Box::new()`)
- Lock contention (use lock-free data structures)
- Expensive computations
- I/O operations

```rust
// BAD: Allocation in callback
|data| {
    let buffer = Vec::new();  // ❌ Don't allocate!
    buffer.extend_from_slice(data);
}

// GOOD: Reuse pre-allocated buffers
|data| {
    let mut buffer = pool.pop()
        .unwrap_or_else(|| Vec::with_capacity(2048));
    buffer.clear();
    buffer.extend_from_slice(data);
    // ... process ...
    pool.push(buffer);
}
```

Use `Arc<SegQueue<T>>` for lock-free cross-thread communication.

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixer_basic() {
        let mut mixer = AudioMixer::new();
        // test implementation
    }

    // Use descriptive test names
    #[test]
    fn test_ring_buffer_fill_ratio() {
        // test implementation
    }
}
```

### Threading and Concurrency

- Use `Arc<AtomicBool>` for simple atomic flags
- Use `Arc<SegQueue<T>>` for lock-free queues
- Use `parking_lot::Mutex` if locks are necessary (faster than std::sync::Mutex)

```rust
use std::sync::atomic::{AtomicBool, Ordering};

pub struct AudioEngine {
    running: Arc<AtomicBool>,
}

impl AudioEngine {
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}
```

### Constants and Magic Numbers

```rust
// Define constants for audio-related values
const DEFAULT_SAMPLE_RATE: u32 = 48000;
const DEFAULT_CHANNELS: u16 = 2;
const DEFAULT_BUFFER_SIZE: usize = 512;
const SOFT_LIMITER_THRESHOLD: f32 = 0.9;
const PEAK_DECAY_RATE: f32 = 0.999;
```

### Comments

Keep comments minimal and focused on "why" not "what":
- Complex audio algorithms may need brief explanations
- Non-obvious performance optimizations
- Workarounds for platform-specific issues

Avoid redundant comments that repeat the code.

## Key Patterns

### Ring Buffer Pattern

For cross-thread audio data transfer:
```rust
pub struct RingBuffer {
    buffer: Vec<f32>,
    write_pos: usize,
    read_pos: usize,
    capacity: usize,
}
```

### Buffer Pool Pattern

Pre-allocate and reuse buffers to avoid reallocation:
```rust
pub struct BufferPool {
    pool: Arc<SegQueue<Vec<f32>>>,
}
```

### Soft Limiting

Use tanh for smooth clipping:
```rust
fn soft_limiter(sample: f32) -> f32 {
    sample.tanh()
}
```

### dB to Linear Conversion

```rust
fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}
```

## Platform-Specific Notes

This project targets Windows with WASAPI. Key considerations:
- Use CPAL for cross-platform audio I/O
- Shared mode is preferred (allows other apps to use audio)
- Target output device: VB-Cable virtual audio cable
- Typical sample rates: 44.1kHz, 48kHz
- Typical bit depths: 32-bit float

## When Working in This Codebase

1. Always run tests before and after changes: `cargo test`
2. Format code: `cargo fmt`
3. Run clippy: `cargo clippy`
4. Never allocate memory in audio callbacks
5. Use lock-free data structures for inter-thread communication
6. Prefer pre-allocation and buffer reuse
7. Document public APIs with doc comments
8. Add tests for new functionality
9. Consider performance implications - audio processing is time-sensitive
