# Antigravity Manager 🚀

<div align="center">

![Platform Support](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS%20%7C%20Docker-blue?style=flat-square)
![Version](https://img.shields.io/badge/version-4.0.11-orange?style=flat-square)
![License](https://img.shields.io/badge/license-CC--BY--NC--SA--4.0-lightgrey?style=flat-square)
![Tauri](https://img.shields.io/badge/Tauri-v2-orange?style=flat-square)
![Backend](https://img.shields.io/badge/Backend-Rust-red?style=flat-square)
![Frontend](https://img.shields.io/badge/Frontend-React-61DAFB?style=flat-square)

**Professional AI Account Management & API Gateway**

Transform your Google Gemini and Anthropic Claude web sessions into standardized API endpoints

[English](#english) | [简体中文](#简体中文) | [Features](#-features) | [Quick Start](#-quick-start) | [Documentation](#-documentation)

</div>

---

## English

### 🎯 What is Antigravity Manager?

Antigravity Manager is a **cross-platform desktop application** that provides enterprise-grade account management and API proxying for Google Gemini and Anthropic Claude. It converts browser sessions into OpenAI-compatible API endpoints, enabling seamless integration with existing AI tools and workflows.

### ✨ Features

#### 🌐 Multi-Account Management

- **Unlimited Account Pool**: Manage multiple Google/Claude accounts with OAuth 2.0
- **Visual Dashboard**: Real-time quota monitoring with color-coded indicators
- **Smart Switching**: Automatic account rotation when quota exhausted
- **Batch Operations**: Import/export accounts, bulk refresh, smart recommendations

#### 🔌 API Proxy Layer

- **OpenAI Compatible**: Drop-in replacement with `/v1/chat/completions` endpoint
- **Multi-Protocol Support**: OpenAI, Anthropic, and Gemini native formats
- **Model Mapping**: Intelligent routing (e.g., GPT-4 → Gemini-2.0-Flash)
- **Streaming Support**: Full SSE (Server-Sent Events) implementation

#### 🛡️ Security & Reliability

- **End-to-End Encryption**: AES-256-GCM for credentials
- **OS Keyring Integration**: Windows Credential Manager, macOS Keychain support
- **Circuit Breaker**: Auto-retry with exponential backoff
- **Rate Limiting**: Per-account, per-model quota protection

#### 🌍 Internationalization

- **12 Languages**: EN, ZH-CN, ZH-TW, AR, JA, KO, PT, RU, TR, VI, ES, MY
- **RTL Support**: Right-to-left layout for Arabic
- **Auto-Detection**: Detects system language preferences

#### 🎨 Multi-Modal Support

- **Image Generation**: Imagen 3 support via OpenAI Images API
- **Vision Models**: 4K image recognition with 100MB payload support
- **Quality Control**: Auto-mapping for HD/2K/Standard resolutions

---

### 🚀 Quick Start

#### Windows

```powershell
# Download installer
Invoke-WebRequest -Uri https://github.com/atnplex/antigravity-manager/releases/latest/download/AntigravityManager-Setup.msi -OutFile setup.msi

# Install
msiexec /i setup.msi
```

#### Linux

```bash
# Ubuntu/Debian (.deb)
wget https://github.com/atnplex/antigravity-manager/releases/latest/download/antigravity-manager_4.0.11_amd64.deb
sudo dpkg -i antigravity-manager_4.0.11_amd64.deb

# AppImage (Universal)
wget https://github.com/atnplex/antigravity-manager/releases/latest/download/AntigravityManager.AppImage
chmod +x AntigravityManager.AppImage
./AntigravityManager.AppImage

# Arch Linux (Homebrew)
brew tap atnplex/antigravity-manager
brew install --cask antigravity-tools
```

#### macOS

```bash
# Homebrew (Recommended)
brew tap atnplex/antigravity-manager
brew install --cask antigravity-tools

# Or download .dmg
# Supports Apple Silicon & Intel
```

#### 🐳 Docker (All Platforms)

```bash
docker run -d \
  --name antigravity-manager \
  -p 8045:8045 \
  -v ~/.antigravity_tools:/root/.antigravity_tools \
  -e API_KEY=sk-your-api-key \
  -e WEB_PASSWORD=your-admin-password \
  -e ALLOW_LAN=1 \
  --restart unless-stopped \
  atnplex/antigravity-manager:latest

# Access: http://localhost:8045
# API Base: http://localhost:8045/v1
```

---

### 💻 Usage Examples

#### Python (OpenAI SDK)

```python
import openai

client = openai.OpenAI(
    api_key="sk-antigravity",
    base_url="http://localhost:8045/v1"
)

response = client.chat.completions.create(
    model="gpt-4",  # Auto-maps to gemini-2.0-flash
    messages=[{"role": "user", "content": "Hello!"}]
)

print(response.choices[0].message.content)
```

#### Claude Code CLI

```bash
export ANTHROPIC_API_KEY="sk-antigravity"
export ANTHROPIC_BASE_URL="http://127.0.0.1:8045"
claude "Write a Python script to analyze CSV files"
```

#### JavaScript (OpenAI SDK)

```javascript
import OpenAI from 'openai';

const client = new OpenAI({
  apiKey: 'sk-antigravity',
  baseURL: 'http://localhost:8045/v1',
});

const response = await client.chat.completions.create({
  model: 'gpt-4',
  messages: [{ role: 'user', content: 'Explain quantum computing' }],
});

console.log(response.choices[0].message.content);
```

---

### 📖 Documentation

- **Installation Guide**: [docs/installation.md](docs/installation.md)
- **Configuration**: [docs/configuration.md](docs/configuration.md)
- **API Reference**: [docs/api-reference.md](docs/api-reference.md)
- **Troubleshooting**: [docs/troubleshooting.md](docs/troubleshooting.md)

---

### 🛠️ Development

```bash
# Clone repository
git clone https://github.com/atnplex/antigravity-manager.git
cd antigravity-manager

# Install dependencies
npm install

# Run development server
npm run tauri dev

# Build for production
npm run tauri build
```

**Tech Stack**:

- Frontend: React 19 + TypeScript + TailwindCSS
- Backend: Rust (Axum + Tokio)
- Desktop: Tauri v2
- Database: SQLite
- I18n: i18next

---

### 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### 📄 License

This project is licensed under the CC-BY-NC-SA-4.0 License - see [LICENSE](LICENSE) for details.

### ⭐ Star History

If you find this project helpful, please consider giving it a star!

---

## 简体中文

### 🎯 什么是 Antigravity Manager?

Antigravity Manager 是一个**跨平台桌面应用程序**,为 Google Gemini 和 Anthropic Claude 提供企业级账号管理和 API 代理服务。它将浏览器会话转换为 OpenAI 兼容的 API 端点,实现与现有 AI 工具和工作流的无缝集成。

### ✨ 核心功能

#### 🌐 多账号管理

- **无限账号池**: 通过 OAuth 2.0 管理多个 Google/Claude 账号
- **可视化仪表盘**: 实时配额监控,彩色状态指示器
- **智能切换**: 配额耗尽时自动轮换账号
- **批量操作**: 导入/导出账号,批量刷新,智能推荐

#### 🔌 API 代理层

- **OpenAI 兼容**: 提供 `/v1/chat/completions` 端点,即插即用
- **多协议支持**: OpenAI、Anthropic 和 Gemini 原生格式
- **模型映射**: 智能路由 (如 GPT-4 → Gemini-2.0-Flash)
- **流式支持**: 完整的 SSE (Server-Sent Events) 实现

#### 🛡️ 安全与可靠性

- **端到端加密**: AES-256-GCM 加密凭据
- **系统密钥环集成**: 支持 Windows 凭据管理器、macOS 钥匙串
- **熔断器**: 自动重试与指数退避
- **速率限制**: 按账号、按模型的配额保护

#### 🌍 国际化

- **12 种语言**: EN, ZH-CN, ZH-TW, AR, JA, KO, PT, RU, TR, VI, ES, MY
- **RTL 支持**: 阿拉伯语从右至左布局
- **自动检测**: 检测系统语言偏好

#### 🎨 多模态支持

- **图像生成**: 通过 OpenAI Images API 支持 Imagen 3
- **视觉模型**: 4K 图像识别,支持 100MB 载荷
- **质量控制**: 自动映射 HD/2K/Standard 分辨率

---

### 🚀 快速开始

#### Windows

```powershell
# 下载安装程序
Invoke-WebRequest -Uri https://github.com/atnplex/antigravity-manager/releases/latest/download/AntigravityManager-Setup.msi -OutFile setup.msi

# 安装
msiexec /i setup.msi
```

#### Linux

```bash
# Ubuntu/Debian (.deb)
wget https://github.com/atnplex/antigravity-manager/releases/latest/download/antigravity-manager_4.0.11_amd64.deb
sudo dpkg -i antigravity-manager_4.0.11_amd64.deb

# AppImage (通用)
wget https://github.com/atnplex/antigravity-manager/releases/latest/download/AntigravityManager.AppImage
chmod +x AntigravityManager.AppImage
./AntigravityManager.AppImage

# Arch Linux (Homebrew)
brew tap atnplex/antigravity-manager
brew install --cask antigravity-tools
```

#### macOS

```bash
# Homebrew (推荐)
brew tap atnplex/antigravity-manager
brew install --cask antigravity-tools

# 或下载 .dmg
# 支持 Apple Silicon 和 Intel
```

#### 🐳 Docker (所有平台)

```bash
docker run -d \
  --name antigravity-manager \
  -p 8045:8045 \
  -v ~/.antigravity_tools:/root/.antigravity_tools \
  -e API_KEY=sk-your-api-key \
  -e WEB_PASSWORD=your-admin-password \
  -e ALLOW_LAN=1 \
  --restart unless-stopped \
  atnplex/antigravity-manager:latest

# 访问: http://localhost:8045
# API Base: http://localhost:8045/v1
```

---

### 💻 使用示例

#### Python (OpenAI SDK)

```python
import openai

client = openai.OpenAI(
    api_key="sk-antigravity",
    base_url="http://localhost:8045/v1"
)

response = client.chat.completions.create(
    model="gpt-4",  # 自动映射到 gemini-2.0-flash
    messages=[{"role": "user", "content": "你好!"}]
)

print(response.choices[0].message.content)
```

#### Claude Code CLI

```bash
export ANTHROPIC_API_KEY="sk-antigravity"
export ANTHROPIC_BASE_URL="http://127.0.0.1:8045"
claude "编写一个 Python 脚本来分析 CSV 文件"
```

---

### 📖 文档

- **安装指南**: [docs/installation.md](docs/installation.md)
- **配置说明**: [docs/configuration.md](docs/configuration.md)
- **API 参考**: [docs/api-reference.md](docs/api-reference.md)
- **故障排除**: [docs/troubleshooting.md](docs/troubleshooting.md)

---

### 🛠️ 开发

```bash
# 克隆仓库
git clone https://github.com/atnplex/antigravity-manager.git
cd antigravity-manager

# 安装依赖
npm install

# 运行开发服务器
npm run tauri dev

# 构建生产版本
npm run tauri build
```

**技术栈**:

- 前端: React 19 + TypeScript + TailwindCSS
- 后端: Rust (Axum + Tokio)
- 桌面: Tauri v2
- 数据库: SQLite
- 国际化: i18next

---

### 🤝 贡献

我们欢迎贡献! 请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解guidelines。

### 📄 许可证

本项目基于 CC-BY-NC-SA-4.0 许可证 - 查看 [LICENSE](LICENSE) 了解详情。

### ⭐ Star 历史

如果您觉得这个项目有帮助,请考虑给它一个 star!

---

<div align="center">

**Made with ❤️ by the Antigravity Team**

[GitHub](https://github.com/atnplex/antigravity-manager) • [Issues](https://github.com/atnplex/antigravity-manager/issues) • [Discussions](https://github.com/atnplex/antigravity-manager/discussions)

</div>
