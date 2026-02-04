# Antigravity Manager

<div align="center">

![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-blue?style=flat-square)
![Version](https://img.shields.io/badge/version-4.0.11--atnplex-orange?style=flat-square)
![License](https://img.shields.io/badge/license-CC--BY--NC--SA--4.0-lightgrey?style=flat-square)

**Professional AI Account & API Management**

Transform Google Gemini and Anthropic Claude sessions into standardized OpenAI-compatible APIs

[Features](#features) • [Installation](#installation) • [Usage](#usage) • [Configuration](#configuration)

</div>

---

## Features

### Multi-Account Management

- Manage unlimited Google Gemini and Anthropic Claude accounts
- Real-time quota monitoring with visual indicators
- Automatic account rotation when quota exhausted
- OAuth 2.0 authentication with secure credential storage

### API Gateway

- **OpenAI-Compatible**: `/v1/chat/completions` endpoint
- **Multi-Protocol**: OpenAI, Anthropic, Gemini native formats
- **Model Mapping**: Route requests intelligently (GPT-4 → Gemini-2.0-Flash)
- **Streaming**: Full Server-Sent Events (SSE) support

### Security

- AES-256-GCM encryption for sensitive credentials
- OS keyring integration (Windows Credential Manager, macOS Keychain)
- Rate limiting and circuit breaker patterns
- Per-account, per-model quota protection

### Advanced Features

- Image generation (Imagen 3) via OpenAI Images API
- 4K vision model support with 100MB payloads
- Context compression for long conversations
- Debug console with real-time logging

---

## Installation

### Windows

```powershell
# Download from releases
Invoke-WebRequest -Uri https://github.com/atnplex/antigravity-manager/releases/latest/download/setup.msi -OutFile setup.msi
msiexec /i setup.msi
```

### Linux

```bash
# AppImage (Universal)
wget https://github.com/atnplex/antigravity-manager/releases/latest/download/AntigravityManager.AppImage
chmod +x AntigravityManager.AppImage
./AntigravityManager.AppImage

# Debian/Ubuntu
wget https://github.com/atnplex/antigravity-manager/releases/latest/download/antigravity-manager_amd64.deb
sudo dpkg -i antigravity-manager_amd64.deb
```

### macOS

```bash
# Homebrew
brew tap atnplex/antigravity-manager
brew install --cask antigravity-tools

# Or download .dmg from releases (Apple Silicon & Intel supported)
```

### Docker

```bash
docker run -d \
  --name antigravity-manager \
  -p 8045:8045 \
  -v ~/.antigravity_tools:/root/.antigravity_tools \
  -e API_KEY=sk-your-secure-key \
  -e WEB_PASSWORD=admin-password \
  -e ALLOW_LAN=1 \
  --restart unless-stopped \
  atnplex/antigravity-manager:latest
```

**Access**: <http://localhost:8045>

---

## Usage

### Python (OpenAI SDK)

```python
import openai

client = openai.OpenAI(
    api_key="sk-antigravity",
    base_url="http://localhost:8045/v1"
)

response = client.chat.completions.create(
    model="gpt-4",
    messages=[{"role": "user", "content": "Hello!"}]
)

print(response.choices[0].message.content)
```

### Claude Code CLI

```bash
export ANTHROPIC_API_KEY="sk-antigravity"
export ANTHROPIC_BASE_URL="http://127.0.0.1:8045"
claude "Write a Python function to parse JSON"
```

### JavaScript/TypeScript

```javascript
import OpenAI from 'openai';

const client = new OpenAI({
  apiKey: 'sk-antigravity',
  baseURL: 'http://localhost:8045/v1',
});

const response = await client.chat.completions.create({
  model: 'gpt-4',
  messages: [{ role: 'user', content: 'Explain async/await' }],
});
```

### Image Generation

```python
response = client.images.generate(
    model="gemini-3-pro-image",
    prompt="A futuristic city with neon lights",
    size="1920x1080",
    quality="hd"
)

# Save image
import base64
with open("output.png", "wb") as f:
    f.write(base64.b64decode(response.data[0].b64_json))
```

---

## Configuration

### Environment Variables

```bash
API_KEY=sk-your-api-key          # Required: API authentication key
WEB_PASSWORD=admin-password      # Optional: Web UI password (defaults to API_KEY)
ALLOW_LAN=1                      # 0=localhost only, 1=LAN accessible
ABV_MAX_BODY_SIZE=104857600      # Max request body size (default: 100MB)
```

### Config File

Location: `~/.antigravity_tools/gui_config.json`

```json
{
  "proxy": {
    "enabled": true,
    "port": 8045,
    "api_key": "sk-your-key",
    "admin_password": "your-password"
  },
  "model_mapping": {
    "gpt-4": "gemini-2.0-flash",
    "gpt-4-vision": "gemini-2.0-flash",
    "claude-3-5-sonnet": "gemini-2.0-flash"
  }
}
```

---

## Development

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

---

## Architecture

```
┌──────────────────┐
│  Client App      │  (Claude Code, NextChat, etc.)
│  OpenAI/Claude   │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Axum Server     │  Authentication & Rate Limiting
│  (Port 8045)     │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Protocol        │  OpenAI ↔ Gemini/Claude conversion
│  Converter       │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Account Pool    │  Smart rotation & quota management
│  Manager         │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Upstream API    │  Google AI / Anthropic
└──────────────────┘
```

---

## Troubleshooting

### macOS: "App is damaged"

```bash
sudo xattr -rd com.apple.quarantine "/Applications/Antigravity Tools.app"
```

### Docker: Port already in use

```bash
# Stop existing container
docker stop antigravity-manager
docker rm antigravity-manager

# Or use different port
docker run -p 8046:8045 ...
```

### Authentication issues

```bash
# Check logs
docker logs antigravity-manager

# Reset password
docker exec -it antigravity-manager cat /root/.antigravity_tools/gui_config.json
```

---

## License

**CC-BY-NC-SA-4.0** - see [LICENSE](LICENSE) for full terms.

This is a non-commercial fork maintained by ATNplex for internal use.

**Original Project**: [lbjlaq/Antigravity-Manager](https://github.com/lbjlaq/Antigravity-Manager)
**Original Author**: [lbjlaq](https://github.com/lbjlaq)
**Modified by**: ATNplex

---

<div align="center">

**Maintained by ATNplex**

[GitHub](https://github.com/atnplex/antigravity-manager) • [Issues](https://github.com/atnplex/antigravity-manager/issues)

</div>
