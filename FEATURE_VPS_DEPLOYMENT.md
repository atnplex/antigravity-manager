# VPS Deployment with HA

This branch adds comprehensive VPS deployment capabilities with High Availability setup.

## Features Added

### 1. VPS Deployment (`deploy-vps.sh`)

- One-click deployment to Debian 12 VPS
- Docker-based deployment
- Cloudflare Tunnel integration
- Tailscale mesh networking
- Automatic HA setup across multiple VPS

### 2. MCP Server Enhancements

- **16 MCP servers** configured out of the box
- Auto-approve MCP server (rule-based approval)
- Dev-tools MCP server (formatters: shfmt, black, prettier)
- Cloudflare MCP integration (Workers, Observability, Browser, AI Gateway)

### 3. Credential Management

- Encrypted credential storage
- Pre-configured token support
- Persistent across restarts
- Helper scripts for easy setup

### 4. High Availability

- Multi-VPS deployment support
- Automatic failover via Cloudflare Tunnel
- Config synchronization between nodes

## Files Added

- `deploy-vps.sh` - VPS deployment orchestration
- `auto-approve-mcp-server/` - Auto-approval MCP server
- `dev-tools-mcp-server/` - Development tools MCP server
- `vps-deployment-package/` - Complete deployment package
- `add-cf-token.sh` - Token configuration helper
- `setup-antigravity-remote.sh` - Remote access setup

## Deployment

See `vps-deployment-package/DEPLOY.md` for complete deployment guide.

## Breaking Changes

None - all features are additive and backward compatible.
