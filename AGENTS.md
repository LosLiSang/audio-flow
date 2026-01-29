# AGENTS.md

Tauri audio mixer app with React frontend and Rust backend. Windows + WASAPI target.

## Build/Test/Lint Commands

```bash
# Frontend (React/TypeScript)
npm run dev              # Start dev server
npm run build            # Build for production
npm run preview          # Preview production build

# Backend (Rust/Tauri)
npm run tauri:dev        # Full app in dev mode
npm run tauri:build      # Build release app
cd src-tauri && cargo test                    # Run all Rust tests
cd src-tauri && cargo test test_name -- --nocapture  # Single test with output
cd src-tauri && cargo fmt                     # Format Rust code
cd src-tauri && cargo clippy -- -D warnings   # Lint Rust code
```

## Project Structure

- `src/` - React/TypeScript frontend with Emotion styled components
- `src-tauri/src/` - Rust backend (Tauri commands, audio engine, device management)
- `src/types.ts` - Shared TypeScript interfaces for devices, routes, peak levels

## Code Style Guidelines

### TypeScript/React

```typescript
// Interfaces for type safety
export interface DeviceInfo {
  id: string
  name: string
  is_input: boolean
}

// Use Emotion for styled components
const AppContainer = styled.div`
  display: flex;
  height: 100vh;
`

// Async Tauri commands with error handling
const loadDevices = async () => {
  try {
    const devices = await invoke<DeviceInfo[]>('list_devices')
    setDevices(devices)
  } catch (error) {
    console.error('Failed to load devices:', error)
  }
}
```

### Rust

```rust
// std imports first, external crates second, local imports last
use std::sync::Arc;
use cpal::traits::DeviceTrait;
use crate::audio::AudioEngine;

// PascalCase for structs, snake_case for functions
pub struct AudioMixer { buffer_size: usize }

// Result<T, Box<dyn std::error::Error>> for fallible functions
pub fn create_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    let device = host.default_input_device()
        .ok_or("No input device found")?;
    Ok(())
}

// Arc<T> for thread-safe sharing
pub struct AudioEngine {
    running: Arc<AtomicBool>,
}
```

## Key Patterns

### Tauri Commands

Define in `src-tauri/src/commands/`:
```rust
#[tauri::command]
pub async fn list_devices(state: State<'_, AppState>) -> Result<Vec<DeviceInfo>, String> {
    // Implementation
}
```

Register in `main.rs`:
```rust
.invoke_handler(tauri::generate_handler![
    commands::list_devices,
    commands::start_engine,
])
```

### Audio Performance

**CRITICAL**: Audio callbacks must be real-time. Avoid:
- Memory allocation (no `Vec::new()`)
- Lock contention (use `Arc<SegQueue<T>>`)
- Expensive computations and I/O

Use pre-allocated buffers and buffer pools.

## When Working Here

1. Run `npm run tauri:dev` for development
2. Test backend: `cd src-tauri && cargo test`
3. Format both: `npm run build` (TypeScript) + `cargo fmt` (Rust)
4. Lint both: Check TypeScript errors + `cargo clippy`
5. Audio code: Never allocate in callbacks, use lock-free queues
6. Frontend: Use TypeScript types from `src/types.ts`, handle Tauri errors
7. Add tests for new Rust functionality in `#[cfg(test)]` modules
