#!/bin/bash
# Helper script to add Cloudflare token to deployment package

set -e

PACKAGE_DIR="$HOME/vps-deployment-package"

echo "=== Add Cloudflare Token to Deployment ==="
echo ""
echo "This will add your CF token to the deployment package"
echo "so you don't have to paste it manually on each VPS."
echo ""

# Get token
read -p "Cloudflare Tunnel Token: " cf_token

if [ -z "$cf_token" ]; then
    echo "Error: Token cannot be empty"
    exit 1
fi

# Create env file
cat > "$PACKAGE_DIR/cloudflare.env" << EOF
# Cloudflare Tunnel Configuration
# This token will be used on both VPS for HA setup
CF_TUNNEL_TOKEN=$cf_token
EOF

chmod 600 "$PACKAGE_DIR/cloudflare.env"

# Re-package
cd "$PACKAGE_DIR"
tar czf antigravity-deploy.tar.gz .

echo ""
echo "✓ Token added to package!"
echo "✓ Package updated: $PACKAGE_DIR/antigravity-deploy.tar.gz"
echo ""
echo "Now transfer to VPS:"
echo "  scp antigravity-deploy.tar.gz vps1:~"
echo "  scp antigravity-deploy.tar.gz vps2:~"
echo ""
echo "Deploy will use pre-configured token automatically!"
