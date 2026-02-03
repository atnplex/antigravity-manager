# Antigravity Manager - Docker Deployment Runbook

## Prerequisites

- Docker and Docker Compose installed
- Debian/WSL environment (or any Linux host)
- Repository cloned: `git clone https://github.com/YOUR_USERNAME/antigravity-manager.git`

---

## Quick Start (Localhost-Only - SECURE)

This is the **default secure mode** - the service will only be accessible from the same machine.

### Step 1: Configure Environment

```bash
cd docker
cp .env.example .env
```

### Step 2: Edit `.env` with Your Secrets

**CRITICAL**: Never commit `.env` to version control! It contains sensitive credentials.

```bash
nano .env  # or vim, code, etc.
```

Set at minimum:

```bash
API_KEY=your-secure-random-api-key-here
WEB_PASSWORD=your-secure-admin-password-here
```

**Generate a secure API key:**

```bash
# Option 1: OpenSSL
openssl rand -hex 32

# Option 2: Python
python3 -c "import secrets; print(secrets.token_hex(32))"
```

### Step 3: Start the Service

```bash
docker compose up -d
```

### Step 4: Verify Localhost-Only Binding

```bash
# Check logs
docker compose logs

# Verify it's listening on 127.0.0.1 only
docker exec antigravity-manager cat /proc/net/tcp | grep 1F6D
# Should show: 0100007F:1F6D (127.0.0.1:8045)
```

### Step 5: Access the Web UI

Open in your browser:

```
http://127.0.0.1:8045
```

Login with:

- **Username**: `admin` (or your configured username)
- **Password**: The `WEB_PASSWORD` you set in `.env`

---

## LAN Mode (Expose to Network - USE WITH CAUTION)

> ⚠️ **SECURITY WARNING**: Only enable LAN mode if you need to access the service from other devices on your network. This exposes the admin interface to your entire local network.

### Step 1: Edit `.env`

```bash
ALLOW_LAN=1
```

### Step 2: Edit `docker-compose.yml`

Change the ports mapping from:

```yaml
ports:
  - "127.0.0.1:8045:8045"
```

To:

```yaml
ports:
  - "8045:8045"
```

### Step 3: Restart the Service

```bash
docker compose down
docker compose up -d
```

### Step 4: Verify 0.0.0.0 Binding

```bash
docker exec antigravity-manager cat /proc/net/tcp | grep 1F6D
# Should show: 00000000:1F6D (0.0.0.0:8045)
```

### Step 5: Access from LAN

Access from other devices:

```
http://<your-server-ip>:8045
```

---

## Common Operations

### View Logs

```bash
docker compose logs -f
```

### Stop the Service

```bash
docker compose down
```

### Update to Latest Image

```bash
docker compose pull
docker compose up -d
```

### Backup Configuration

```bash
# Backup the persistent volume
tar -czf antigravity-backup-$(date +%Y%m%d).tar.gz ~/.antigravity_tools
```

### Reset to Factory Defaults

```bash
docker compose down
rm -rf ~/.antigravity_tools
docker compose up -d
```

---

## Security Best Practices

### 🔐 Secrets Management

1. **Never** commit `.env` to version control
2. **Always** use strong, random values for `API_KEY` and `WEB_PASSWORD`
3. **Store** `.env` securely (encrypted filesystem, password manager, etc.)
4. **Rotate** credentials periodically

### 🌐 Network Security

1. **Default**: Use localhost-only mode unless you specifically need LAN access
2. **Firewall**: If using LAN mode, configure firewall rules to restrict access
3. **VPN/Tunnel**: Consider using Tailscale or WireGuard instead of exposing to LAN
4. **Reverse Proxy**: Use nginx/Caddy with HTTPS if exposing externally

### 🔒 Container Security

1. **Volume Permissions**: Ensure `~/.antigravity_tools` has appropriate permissions
2. **User Isolation**: Consider running container as non-root user (future enhancement)
3. **Network Isolation**: Use Docker networks to isolate from other services

---

## Troubleshooting

### Port Already in Use

```bash
# Find what's using port 8045
sudo lsof -i :8045
# or
sudo netstat -tulpn | grep 8045

# Stop the conflicting service or change the port in docker-compose.yml
```

### Container Won't Start

```bash
# Check logs for errors
docker compose logs

# Verify environment variables are set
docker compose config

# Check if image exists
docker images | grep antigravity
```

### Can't Access Web UI

```bash
# Verify container is running
docker ps | grep antigravity

# Check network binding
docker exec antigravity-manager cat /proc/net/tcp | grep 1F6D

# Test from localhost
curl http://127.0.0.1:8045

# Check firewall (if using LAN mode)
sudo ufw status
```

### Permission Denied on Volume

```bash
# Fix permissions on data directory
chmod 700 ~/.antigravity_tools
```

---

## Environment Variables Reference

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `API_KEY` | Yes | - | API key for proxy authentication |
| `WEB_PASSWORD` | Yes | - | Admin panel password |
| `LOG_LEVEL` | No | `info` | Logging level: `debug`, `info`, `warn`, `error` |
| `ALLOW_LAN` | No | `0` | Enable LAN binding: `0`=localhost, `1`=0.0.0.0 |

---

## Advanced Configuration

### Using External Reverse Proxy

If using nginx or Caddy in front:

**docker-compose.yml:**

```yaml
ports:
  - "127.0.0.1:8045:8045"  # Only expose to localhost
```

**Nginx example:**

```nginx
server {
    listen 80;
    server_name antigravity.yourdomain.com;

    location / {
        proxy_pass http://127.0.0.1:8045;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### Resource Limits

Add to `docker-compose.yml`:

```yaml
deploy:
  resources:
    limits:
      cpus: '2.0'
      memory: 2G
    reservations:
      cpus: '0.5'
      memory: 512M
```

---

## Support

- **Issues**: <https://github.com/YOUR_USERNAME/antigravity-manager/issues>
- **Upstream**: <https://github.com/lbjlaq/Antigravity-Manager>

---

**Remember**: This is a hardened fork. Auto-updates are **disabled** by design for security and stability.
