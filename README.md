# Audio Flow - Windows 音频混音器

基于 Rust + Tauri 2.0 + React 的 Windows 音频混音应用，支持 WASAPI 和 VB-Cable 虚拟音频设备。

## 功能特性

- ✅ 多音频设备输入/输出
- ✅ VB-Cable 虚拟设备支持
- ✅ 实时音频混音
- ✅ 增益控制（dB）
- ✅ 音频路由配置
- ✅ 实时 VU 表显示
- ✅ 低延迟（20-50ms）
- ✅ 配置持久化

## 技术栈

**后端：**
- Rust
- Tauri 2.0
- CPAL (WASAPI)
- Crossbeam (无锁队列)
- Tokio (异步运行时)

**前端：**
- React 18
- TypeScript
- Emotion (CSS-in-JS)
- Vite

## 安装依赖

### 安装 Rust

```bash
# Windows 上安装 Rust
# 访问 https://rustup.rs/ 下载安装程序

# 验证安装
rustc --version
cargo --version
```

### 安装 Node.js

```bash
# 访问 https://nodejs.org/ 下载安装程序
# 或使用 nvm-windows: https://github.com/coreybutler/nvm-windows

# 验证安装
node --version
npm --version
```

### 安装项目依赖

```bash
# 安装 Tauri CLI
cargo install tauri-cli

# 安装 npm 依赖
npm install
```

## 开发

### 运行开发服务器

```bash
npm run tauri dev
```

这将：
1. 启动 Vite 开发服务器（前端）
2. 编译 Rust 代码（后端）
3. 打开 Tauri 应用窗口

### 构建发布版本

```bash
npm run tauri build
```

构建产物位于：
- Windows: `src-tauri/target/release/bundle/msi/` 或 `nsis/`

## 使用说明

1. **启动应用**后，点击"刷新设备"按钮检测所有音频设备
2. **添加路由**：从输入设备选择要混音的音频源，输出设备选择 VB-Cable
3. **调整增益**：拖动滑块调整每个路由的增益（-60dB 到 +12dB）
4. **启动混音**：点击"启动"按钮开始音频处理
5. **监控电平**：查看实时 VU 表监控音频电平

## VB-Cable 安装

如果系统未安装 VB-Cable，请访问：
https://vb-audio.com/Cable/

安装后，VB-Cable 会作为虚拟音频输出设备出现在应用中。

## 项目结构

```
audio-flow/
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── audio/          # 音频处理模块
│   │   ├── commands/       # Tauri 命令接口
│   │   ├── config/         # 配置管理
│   │   ├── main.rs         # 应用入口
│   │   └── lib.rs
│   └── Cargo.toml
├── src/                   # React 前端
│   ├── components/         # UI 组件
│   ├── styles/            # 全局样式
│   ├── App.tsx            # 主应用组件
│   └── main.tsx
├── package.json
├── Cargo.toml
└── tsconfig.json
```

## 故障排查

### 编译错误

```bash
# 清理并重新构建
cargo clean
npm run tauri build
```

### 设备访问被拒绝

确保应用有足够的权限访问音频设备。在 Windows 上，可能需要以管理员身份运行。

### 没有检测到设备

- 确保音频设备已正确连接
- 检查 Windows 音频设置中的设备状态
- 点击"刷新设备"按钮重新扫描

### 音频延迟过高

- 在 `src-tauri/src/audio/engine.rs` 中调整缓冲区大小
- 使用较小的缓冲区（如 256 帧）可降低延迟

## 性能优化

- 使用 `release` 模式编译以获得最佳性能
- 在 Tauri 配置中启用 LTO（链接时优化）
- 音频回调中避免内存分配
- 使用无锁数据结构

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

## 致谢

- [Tauri](https://tauri.app/) - 跨平台应用框架
- [CPAL](https://github.com/RustAudio/cpal) - Rust 音频 I/O 库
- [React](https://react.dev/) - UI 框架
- [VB-Audio](https://vb-audio.com/) - 虚拟音频设备
