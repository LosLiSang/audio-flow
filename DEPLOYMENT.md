# Audio Flow - 完整部署指南

## ✅ 项目状态：核心功能完成，可运行

---

## 📦 项目内容

- ✅ 完整的 Tauri 项目结构
- ✅ Rust 后端音频引擎（可编译和运行）
- ✅ React 前端 UI
- ✅ 设备管理功能
- ✅ 音频路由控制
- ✅ 实时峰值检测
- ✅ 增益控制
- ✅ 配置持久化

---

## 🚀 在 Windows 上运行（推荐方式）

### 前置要求

1. **Windows 10/11** - 本项目专为 Windows WASAPI 设计
2. **Rust** - 从 https://rustup.rs/ 下载安装
3. **Node.js 18+** - 从 https://nodejs.org/ 下载安装
4. **VB-Cable** - 从 https://vb-audio.com/Cable/ 安装虚拟音频设备

### 安装步骤

```bash
# 1. 进入项目目录
cd /path/to/audio-flow

# 2. 安装 Tauri CLI（仅需一次）
cargo install tauri-cli

# 3. 安装 npm 依赖
npm install

# 4. 启动开发服务器
npm run tauri dev
```

首次运行会编译 Rust 代码，需要 5-10 分钟。

### 构建发布版本

```bash
npm run tauri build
```

安装包位于：
- MSI: `src-tauri/target/release/bundle/msi/`
- NSIS: `src-tauri/target/release/bundle/nsis/`

---

## 🐧 在 Linux 上测试音频引擎

项目包含一个独立的测试程序，可以在 Linux 上验证核心音频逻辑：

```bash
# 进入测试目录
cd tests/audio_engine

# 运行测试程序
cargo run
```

这会：
1. 列出所有可用的音频设备
2. 验证音频引擎的核心功能

---

## 🎯 使用说明

### 基本操作

1. **启动应用**
   - 应用启动后，左侧显示所有音频设备
   - VB-Cable 设备会特殊标记

2. **添加音频路由**
   - 点击"添加路由"按钮
   - 选择输入设备（如麦克风）
   - 选择输出设备（如 VB-Cable）

3. **调整增益**
   - 拖动增益滑块（-60dB 到 +12dB）
   - 实时调整每个路由的音量

4. **启动混音**
   - 点击"启动"按钮开始音频处理
   - 右下角 VU 表显示实时音频电平

5. **监控音频**
   - 绿色：正常电平
   - 黄色：接近过载
   - 红色：过载警告

### VB-Cable 使用

1. 安装 VB-Cable 后，它会在 Windows 声音设置中显示为输出设备
2. 在 Audio Flow 中，将音频路由到 VB-Cable
3. 在其他应用（如 OBS、录音软件）中选择 VB-Cable 输入
4. 这样就可以录制混音后的音频

---

## 🔧 故障排查

### 编译错误

```bash
# 清理缓存
cd src-tauri
cargo clean

# 重新编译
cargo check
```

### 找不到音频设备

1. 检查 Windows 声音设置中的设备状态
2. 确保设备已启用
3. 以管理员身份运行应用

### 音频没有声音

1. 确认已点击"启动"按钮
2. 检查路由是否已添加
3. 确认增益不为 -inf dB
4. 检查输出设备音量

### VB-Cable 不工作

1. 确认 VB-Cable 已正确安装
2. 重启 Windows 声音服务
3. 检查 VB-Cable 是否被其他应用占用

### 延迟过高

编辑 `src-tauri/src/audio/engine.rs`，调整缓冲区大小：

```rust
// 当前: 512 帧 (~10.6ms @ 48kHz)
// 更低延迟: 256 帧 (~5.3ms @ 48kHz)
// 更稳定: 1024 帧 (~21.3ms @ 48kHz)
```

---

## 📊 性能目标

| 指标 | 目标 | 当前状态 |
|------|------|----------|
| 延迟 | 20-50ms | ✅ 已实现 |
| CPU 使用率 | < 10% | ⏳ 待实际测试 |
| 内存占用 | < 100MB | ⏳ 待实际测试 |
| 启动时间 | < 5s | ⏳ 待实际测试 |

---

## 📚 文档参考

| 文档 | 内容 |
|------|------|
| `QUICKSTART.md` | 快速入门指南 |
| `README.md` | 项目说明 |
| `PROJECT_SUMMARY.md` | 项目总结 |
| `AGENTS.md` | 开发指南 |
| `audio_mixer_guide.md` | 技术架构详解 |
| `implementation_guide.md` | 实现步骤和故障排查 |

---

## 🎓 学习资源

### 核心概念

1. **WASAPI** - Windows 音频会话 API
2. **CPAL** - 跨平台音频 I/O 库
3. **Tauri** - Rust + Web 前端的应用框架
4. **音频混音** - 多路音频信号相加和归一化

### 代码结构

```
Audio Engine (Rust)
├── DeviceManager - 设备枚举和管理
├── AudioMixer   - 混音算法
├── StreamConfig - 音频流配置
└── PeakDetector - 峰值检测

UI (React + TypeScript)
├── DeviceList - 设备列表组件
├── RoutePanel - 路由配置面板
└── VUMeter - VU 表显示
```

---

## 🚀 下一步开发

### 短期目标

1. **添加测试**
   - 单元测试
   - 集成测试
   - 性能测试

2. **增强功能**
   - EQ 均衡器
   - 压缩器
   - 噪声门

3. **改进 UI**
   - 音频波形可视化
   - 预设配置
   - 主题切换

### 长期目标

1. **插件系统** - 支持第三方效果器
2. **录音功能** - 直接录制混音输出
3. **网络音频** - 支持 RTP/RTSP 流
4. **多设备同步** - 时钟同步和漂移补偿

---

## 🤝 贡献指南

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 推送到分支
5. 创建 Pull Request

---

## 📄 许可证

MIT License

---

## 📞 获取帮助

- 查看项目文档
- 检查 Issues
- 阅读 `implementation_guide.md`

---

**祝使用愉快！** 🎉

*最后更新：2025-01-28*
