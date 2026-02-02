#!/bin/bash
# Deploy Antigravity Manager to VPS (Debian 12)
# Supports: Docker deployment, Cloudflare Tunnel, Tailscale

set -e

VPS_NAME="${1:-vps1}"
CONFIG_DIR="/etc/antigravity"
DATA_DIR="/var/lib/antigravity"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Antigravity Manager VPS Deployment   ║${NC}"
echo -e "${BLUE}║  Host: $VPS_NAME                        ${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"

# Install Docker
install_docker() {
    if ! command -v docker &> /dev/null; then
        echo "Installing Docker..."
        curl -fsSL https://get.docker.com | sh
        sudo systemctl enable docker
        sudo systemctl start docker
        sudo usermod -aG docker $USER
    fi
    echo -e "${GREEN}✓ Docker installed${NC}"
}

# Install Tailscale
install_tailscale() {
    if ! command -v tailscale &> /dev/null; then
        echo "Installing Tailscale..."
        curl -fsSL https://tailscale.com/install.sh | sh
    fi

    sudo tailscale up
    TAILSCALE_IP=$(tailscale ip -4)
    echo -e "${GREEN}✓ Tailscale: $TAILSCALE_IP${NC}"
}

# Setup Cloudflare Tunnel
setup_cloudflare() {
    echo -e "\n${BLUE}Cloudflare Tunnel Setup${NC}"
    read -p "CF Tunnel Token (both VPS use same token for HA): " cf_token

    if [ -n "$cf_token" ]; then
        sudo mkdir -p "$CONFIG_DIR"
        echo "CF_TUNNEL_TOKEN=$cf_token" | sudo tee "$CONFIG_DIR/cloudflare.env" > /dev/null
        sudo chmod 600 "$CONFIG_DIR/cloudflare.env"
        echo -e "${GREEN}✓ Token saved${NC}"
    fi
}

# Deploy via Docker
deploy_docker() {
    echo -e "\n${BLUE}Deploying Antigravity Manager${NC}"

    sudo mkdir -p "$DATA_DIR"

    # Stop existing container
    sudo docker stop antigravity-manager 2>/dev/null || true
    sudo docker rm antigravity-manager 2>/dev/null || true

    # Pull latest
    sudo docker pull lbjlaq/antigravity-manager:latest

    # Run
    sudo docker run -d \
        --name antigravity-manager \
        --restart unless-stopped \
        -p 8045:8045 \
        -e API_KEY="${API_KEY:-$(openssl rand -hex 16)}" \
        -e WEB_PASSWORD="${WEB_PASSWORD:-$(openssl rand -hex 16)}" \
        --env-file "$CONFIG_DIR/cloudflare.env" \
        -v "$DATA_DIR:/root/.antigravity_tools" \
        lbjlaq/antigravity-manager:latest

    echo -e "${GREEN}✓ Container started${NC}"

    # Show credentials
    API_KEY=$(sudo docker exec antigravity-manager cat /root/.antigravity_tools/gui_config.json 2>/dev/null | grep api_key | cut -d'"' -f4)
    echo ""
    echo "Save these credentials:"
    echo "  API Key: $API_KEY"
    echo "  Access: http://$(hostname -I | awk '{print $1}'):8045"
}

# Sync configuration
sync_config() {
    echo -e "\n${BLUE}Config Sync${NC}"
    read -p "Sync from VPS (vps1 or vps2)? [Enter to skip]: " source_vps

    if [ -n "$source_vps" ]; then
        rsync -avz "$source_vps:$DATA_DIR/" "$DATA_DIR/"
        echo -e "${GREEN}✓ Config synced from $source_vps${NC}"
    fi
}

# Main
main() {
    install_docker

    echo -e "\n${BLUE}Remote Access Method:${NC}"
    echo "  1) Tailscale"
    echo "  2) Cloudflare Tunnel"
    echo "  3) Both (HA)"
    read -p "Choice [1-3]: " choice

    case $choice in
        1) install_tailscale ;;
        2) setup_cloudflare ;;
        3)
            install_tailscale
            setup_cloudflare
            ;;
    esac

    sync_config
    deploy_docker

    echo -e "\n${GREEN}╔════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  ✓ Deployment Complete!               ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
    echo ""
    echo "Next: Deploy on second VPS using same CF token"
}

main
