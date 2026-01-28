# Audio Flow - 快速入门指南

## 🚀 立即开始

### 1. 安装必要工具

**Windows 用户：**

```bash
# 安装 Rust（如果还没有）
# 访问 https://rustup.rs/ 下载 rustup-init.exe 并运行

# 安装 Node.js（如果还没有）
# 访问 https://nodejs.org/ 下载 LTS 版本并安装

# 验证安装
rustc --version
node --version
```

### 2. 安装项目依赖

```bash
# 进入项目目录
cd /root/code/github/audio-flow

# 安装 Tauri CLI（可能需要几分钟）
cargo install tauri-cli

# 安装 npm 依赖
npm install
```

### 3. 运行开发服务器

```bash
npm run tauri dev
```

首次运行需要编译 Rust 代码，可能需要 5-10 分钟。

编译完成后，应用窗口会自动打开。

### 4. 基本使用

1. **检测设备**：点击左侧"刷新设备"按钮
2. **查看设备**：左侧面板显示所有音频设备（VB-Cable 会高亮显示）
3. **添加路由**：点击"添加路由"按钮创建输入→输出映射
4. **调整增益**：拖动滑块调整音频增益（-60dB 到 +12dB）
5. **启动混音**：点击"启动"按钮开始音频处理
6. **监控电平**：右下角 VU 表显示实时音频电平

## 📋 项目结构速览

```
audio-flow/
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── audio/          # 音频处理
│   │   ├── commands/       # Tauri 命令
│   │   ├── config/         # 配置管理
│   │   ├── main.rs
│   │   └── lib.rs
│   └── Cargo.toml
├── src/                   # React 前端
│   ├── components/         # UI 组件
│   ├── styles/            # 样式
│   ├── App.tsx
│   └── main.tsx
├── package.json
├── Cargo.toml
└── README.md
```

## 🔧 构建发布版本

```bash
npm run tauri build
```

构建产物：
- MSI 安装包：`src-tauri/target/release/bundle/msi/`
- NSIS 安装包：`src-tauri/target/release/bundle/nsis/`

## ⚡ 性能优化

项目已配置以下优化：
- ✅ LTO（链接时优化）
- ✅ 最高的优化级别（opt-level = 3）
- ✅ 单代码生成单元
- ✅ 符号表移除

## 🐛 故障排查

### 编译失败

```bash
# 清理并重新编译
cargo clean
npm run tauri dev
```

### 找不到设备

- 确保音频设备已连接
- 检查 Windows 声音设置
- 以管理员身份运行应用

### 音频延迟高

编辑 `src-tauri/src/audio/engine.rs`，调整缓冲区大小：

```rust
// 当前: 512 帧 (~10.6ms @ 48kHz)
// 更低延迟: 256 帧 (~5.3ms @ 48kHz)
// 更稳定: 1024 帧 (~21.3ms @ 48kHz)
buffer_size: 512,
```

## 📚 下一步

1. **阅读完整文档**：查看 `README.md` 了解所有功能
2. **自定义配置**：编辑 `src-tauri/src/audio/engine.rs` 调整音频参数
3. **添加功能**：参考 `implementation_guide.md` 扩展功能
4. **性能测试**：使用 Windows 任务管理器监控 CPU 使用率

## 💡 提示

- 开发时使用 `npm run tauri dev` 可以热重载前端代码
- 后端 Rust 代码修改需要重新编译（Ctrl+C 停止，重新运行命令）
- 配置文件保存在：`%APPDATA%\audioflow\Audio Flow\config.toml`

## 🎯 目标

- ✅ 低延迟（20-50ms）
- ✅ 多设备混音
- ✅ VB-Cable 支持
- ✅ 可视化监控
- ✅ 配置持久化

## 📞 支持

- 查看代码注释了解实现细节
- 阅读 `AGENTS.md` 了解代码规范
- 查看 `implementation_guide.md` 了解架构设计

祝使用愉快！🎉
