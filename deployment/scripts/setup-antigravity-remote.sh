#!/bin/bash
# One-Click Setup for Antigravity Manager Remote Access
# Supports: Cloudflare Tunnel, Tailscale, Encrypted Credentials

set -e

KEYRING_SERVICE="antigravity-manager"
CONFIG_DIR="$HOME/.antigravity-manager"
ENV_FILE="$CONFIG_DIR/.env"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Antigravity Manager Remote Setup     ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Create config directory
mkdir -p "$CONFIG_DIR"

# Function to save encrypted credential
save_credential() {
    local key="$1"
    local value="$2"

    # Try secret-tool (Linux)
    if command -v secret-tool &> /dev/null; then
        echo "$value" | secret-tool store --label="$key" service "$KEYRING_SERVICE" key "$key" 2>/dev/null || {
            echo -e "${YELLOW}⚠ Keyring not available, saving to .env (less secure)${NC}"
            echo "$key=$value" >> "$ENV_FILE"
        }
    else
        # Fallback to .env file
        echo "$key=$value" >> "$ENV_FILE"
        chmod 600 "$ENV_FILE"
    fi
}

# Function to load credential
load_credential() {
    local key="$1"

    if command -v secret-tool &> /dev/null; then
        secret-tool lookup service "$KEYRING_SERVICE" key "$key" 2>/dev/null || \
        grep "^$key=" "$ENV_FILE" 2>/dev/null | cut -d= -f2-
    else
        grep "^$key=" "$ENV_FILE" 2>/dev/null | cut -d= -f2-
    fi
}

# Auto-detect existing configuration
detect_config() {
    echo -e "\n${BLUE}🔍 Detecting existing configuration...${NC}"

    local tailscale_installed=false
    local tailscale_running=false
    local cloudflared_installed=false

    if command -v tailscale &> /dev/null; then
        tailscale_installed=true
        if tailscale status &> /dev/null; then
            tailscale_running=true
            TAILSCALE_IP=$(tailscale ip -4 2>/dev/null)
        fi
    fi

    if command -v cloudflared &> /dev/null; then
        cloudflared_installed=true
    fi

    echo -e "  Tailscale: $([ "$tailscale_installed" = true ] && echo "${GREEN}✓ Installed${NC}" || echo "Not installed")"
    [ "$tailscale_running" = true ] && echo -e "    IP: ${GREEN}$TAILSCALE_IP${NC}"
    echo -e "  Cloudflared: $([ "$cloudflared_installed" = true ] && echo "${GREEN}✓ Installed${NC}" || echo "Not installed")"
}

# Setup Tailscale
setup_tailscale() {
    echo -e "\n${BLUE}═══ Tailscale Setup ═══${NC}"

    if ! command -v tailscale &> /dev/null; then
        echo "Installing Tailscale..."
        curl -fsSL https://tailscale.com/install.sh | sh
    fi

    echo "Starting Tailscale..."
    if ! tailscale status &> /dev/null 2>&1; then
        sudo tailscale up
    fi

    TAILSCALE_IP=$(tailscale ip -4)
    echo -e "${GREEN}✓ Tailscale running${NC}"
    echo -e "  IP: ${GREEN}$TAILSCALE_IP${NC}"
    echo -e "  Access Antigravity: ${GREEN}http://$TAILSCALE_IP:8045${NC}"

    save_credential "TAILSCALE_IP" "$TAILSCALE_IP"
}

# Setup Cloudflare Tunnel
setup_cloudflare() {
    echo -e "\n${BLUE}═══ Cloudflare Tunnel Setup ═══${NC}"
    echo ""
    echo "Choose mode:"
    echo "  1) Quick Tunnel (temporary URL, no account needed)"
    echo "  2) Named Tunnel (permanent URL, requires CF account)"
    read -p "Mode [1/2]: " cf_mode

    if [ "$cf_mode" = "2" ]; then
        echo ""
        echo "To create a named tunnel:"
        echo "  1. Go to: https://dash.cloudflare.com/zero-trust/tunnels"
        echo "  2. Click 'Create a tunnel'"
        echo "  3. Copy the token"
        echo ""
        read -p "Paste tunnel token: " cf_token

        if [ -n "$cf_token" ]; then
            save_credential "CF_TUNNEL_TOKEN" "$cf_token"
            echo -e "${GREEN}✓ Token saved (encrypted)${NC}"
            echo ""
            echo "To enable in Antigravity Manager:"
            echo "  1. Open Settings → Cloudflare Tunnel"
            echo "  2. Mode: Auth"
            echo "  3. Token will auto-load from saved credentials"
        fi
    else
        echo -e "${GREEN}✓ Quick Tunnel mode${NC}"
        echo ""
        echo "To enable in Antigravity Manager:"
        echo "  1. Open Settings → Cloudflare Tunnel"
        echo "  2. Mode: Quick"
        echo "  3. Click 'Start Tunnel'"
        echo "  4. Copy the generated URL"
    fi
}

# Setup API Keys
setup_api_keys() {
    echo -e "\n${BLUE}═══ API Keys Setup ═══${NC}"
    echo ""
    echo "Configure API keys for MCP servers (optional):"
    echo ""

    # GitHub
    EXISTING_GITHUB=$(load_credential "GITHUB_TOKEN")
    if [ -n "$EXISTING_GITHUB" ]; then
        echo -e "  GitHub: ${GREEN}✓ Already configured${NC}"
    else
        read -p "GitHub PAT (Enter to skip): " github_token
        [ -n "$github_token" ] && save_credential "GITHUB_TOKEN" "$github_token"
    fi

    # Cloudflare
    EXISTING_CF_API=$(load_credential "CLOUDFLARE_API_TOKEN")
    if [ -n "$EXISTING_CF_API" ]; then
        echo -e "  Cloudflare: ${GREEN}✓ Already configured${NC}"
    else
        read -p "Cloudflare API Token (Enter to skip): " cf_api_token
        [ -n "$cf_api_token" ] && save_credential "CLOUDFLARE_API_TOKEN" "$cf_api_token"
    fi

    # Brave
    EXISTING_BRAVE=$(load_credential "BRAVE_API_KEY")
    if [ -n "$EXISTING_BRAVE" ]; then
        echo -e "  Brave: ${GREEN}✓ Already configured${NC}"
    else
        read -p "Brave Search API Key (Enter to skip): " brave_key
        [ -n "$brave_key" ] && save_credential "BRAVE_API_KEY" "$brave_key"
    fi
}

# Generate shell rc snippet
generate_rc_snippet() {
    cat > "$CONFIG_DIR/env.sh" << 'EOF'
# Antigravity Manager Environment
# Auto-generated - source this in your ~/.bashrc or ~/.zshrc

# Load credentials from keyring or .env
load_antigravity_env() {
    local config_dir="$HOME/.antigravity-manager"

    if [ -f "$config_dir/.env" ]; then
        export $(cat "$config_dir/.env" | grep -v '^#' | xargs)
    fi

    # Load from keyring if available
    if command -v secret-tool &> /dev/null; then
        for key in GITHUB_TOKEN CLOUDFLARE_API_TOKEN BRAVE_API_KEY PERPLEXITY_API_KEY; do
            val=$(secret-tool lookup service antigravity-manager key "$key" 2>/dev/null)
            [ -n "$val" ] && export "$key=$val"
        done
    fi
}

load_antigravity_env
EOF

    echo -e "\n${GREEN}✓ Environment loader created${NC}"
    echo -e "  Add to ~/.bashrc: ${BLUE}source $CONFIG_DIR/env.sh${NC}"
}

# Main menu
main_menu() {
    detect_config

    echo -e "\n${BLUE}What would you like to set up?${NC}"
    echo "  1) Tailscale (recommended)"
    echo "  2) Cloudflare Tunnel"
    echo "  3) Both (maximum reliability)"
    echo "  4) API Keys only"
    echo "  5) Exit"
    read -p "Choice [1-5]: " choice

    case $choice in
        1)
            setup_tailscale
            setup_api_keys
            ;;
        2)
            setup_cloudflare
            setup_api_keys
            ;;
        3)
            setup_tailscale
            setup_cloudflare
            setup_api_keys
            ;;
        4)
            setup_api_keys
            ;;
        5)
            echo "Exiting..."
            exit 0
            ;;
        *)
            echo "Invalid choice"
            exit 1
            ;;
    esac

    generate_rc_snippet

    echo -e "\n${GREEN}╔════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  ✓ Setup Complete!                    ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Source environment: source $CONFIG_DIR/env.sh"
    echo "  2. Restart Antigravity Manager"
    echo "  3. Access from anywhere using your chosen method"
}

# Run
main_menu
