# VPS Deployment Guide

## Step 1: Create Cloudflare Tunnel

**Go to:** <https://dash.cloudflare.com/zero-trust/tunnels>

1. Click **"Create a tunnel"**
2. Name: `antigravity-ha`
3. Environment: **Docker**
4. **Copy the tunnel token** (starts with `eyJ...`)

**Configure public hostname:**

- Subdomain: `api` (or your choice)
- Domain: Your Cloudflare domain
- Service: `http://localhost:8045`

Save and copy the token!

## Step 2: Transfer to VPS

```bash
# Package everything
cd ~/vps-deployment-package
tar czf antigravity-deploy.tar.gz .

# Transfer to VPS1
scp antigravity-deploy.tar.gz vps1:~

# Transfer to VPS2
scp antigravity-deploy.tar.gz vps2:~
```

## Step 3: Deploy to VPS1

```bash
ssh vps1

# Extract
tar xzf antigravity-deploy.tar.gz

# Run deployment
./deploy-vps.sh

# When prompted:
# - Choice: 3 (Both - CF + Tailscale)
# - Paste CF Tunnel Token
# - Wait for completion
```

## Step 4: Deploy to VPS2

```bash
ssh vps2

# Extract
tar xzf antigravity-deploy.tar.gz

# Run deployment
./deploy-vps.sh

# When prompted:
# - Choice: 3 (Both)
# - Paste SAME CF Tunnel Token
# - Sync from: vps1
```

## Step 5: Verify HA

**Check Cloudflare Dashboard:**

- Should show 2 connectors (VPS1 + VPS2)
- Both should be "Healthy"

**Test failover:**

```bash
# On VPS1
sudo docker stop antigravity-manager

# Access should still work via CF tunnel (routes to VPS2)
curl https://api.yourdomain.com/health

# Restart VPS1
sudo docker start antigravity-manager
```

## Step 6: Configure Clients

**Update all client configs to use CF tunnel:**

**Windows/WSL: `~/.antigravity-server/data/User/mcp.json`**

```json
{
  "servers": {
    "remote-manager": {
      "url": "https://api.yourdomain.com/mcp",
      "type": "http"
    }
  }
}
```

**Or use Tailscale IP:**

```
http://100.x.x.x:8045/mcp
```

## Troubleshooting

**View logs:**

```bash
ssh vps1
sudo docker logs -f antigravity-manager
```

**Restart:**

```bash
sudo docker restart antigravity-manager
```

**Update:**

```bash
sudo docker pull lbjlaq/antigravity-manager:latest
sudo docker restart antigravity-manager
```
