#!/bin/bash
# Retired before any side effects; kept for source context only.
printf '%s\n' 'Retired deployment helper. Use docs/architecture/HOSTING.md and ADR-130; no deployment was performed.' >&2
exit 2

# SSL Certificate Setup for qorsync.dev and qorsync.dev
# Run on Monad server

set -e

echo "╔════════════════════════════════════════════════════════╗"
echo "║   SSL CERTIFICATE SETUP - DEMIURGE DOMAINS            ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

DOMAINS=("qorsync.dev" "qorsync.dev" "qorsync.dev" "qorsync.dev")
EMAIL="[retired contact]"

echo -e "${BLUE}📋 Prerequisites Check${NC}"
echo ""

# Check if domains point to this server
echo -e "${YELLOW}⚠${NC}  Ensure DNS records are configured:"
echo "   - qorsync.dev A → $(curl -s ifconfig.me)"
echo "   - qorsync.dev A → $(curl -s ifconfig.me)"
echo "   - qorsync.dev A → $(curl -s ifconfig.me)"
echo "   - qorsync.dev A → $(curl -s ifconfig.me)"
echo ""

read -p "Press Enter when DNS is configured, or Ctrl+C to cancel..."

# Create directories
echo -e "${BLUE}[1/5] Creating directories...${NC}"
sudo mkdir -p /etc/nginx/ssl/qorsync.dev
sudo mkdir -p /etc/nginx/ssl/qorsync.dev
sudo mkdir -p /var/www/certbot
sudo chown -R ubuntu:ubuntu /var/www/certbot
echo -e "${GREEN}✓${NC} Directories created"

# Stop nginx temporarily for certbot
echo -e "${BLUE}[2/5] Stopping Nginx...${NC}"
sudo systemctl stop nginx 2>/dev/null || docker compose -f /data/Demiurge-Blockchain/docker/docker-compose.production.yml stop nginx 2>/dev/null || true
echo -e "${GREEN}✓${NC} Nginx stopped"

# Obtain certificates
echo -e "${BLUE}[3/5] Obtaining SSL certificates...${NC}"
echo ""

# qorsync.dev
echo -e "${YELLOW}Requesting certificate for qorsync.dev...${NC}"
sudo certbot certonly --standalone \
    --preferred-challenges http \
    -d qorsync.dev \
    -d qorsync.dev \
    --email "$EMAIL" \
    --agree-tos \
    --non-interactive \
    --keep-until-expiring || {
    echo -e "${RED}✗${NC} Failed to obtain certificate for qorsync.dev"
    exit 1
}

# qorsync.dev
echo -e "${YELLOW}Requesting certificate for qorsync.dev...${NC}"
sudo certbot certonly --standalone \
    --preferred-challenges http \
    -d qorsync.dev \
    -d qorsync.dev \
    --email "$EMAIL" \
    --agree-tos \
    --non-interactive \
    --keep-until-expiring || {
    echo -e "${RED}✗${NC} Failed to obtain certificate for qorsync.dev"
    exit 1
}

echo -e "${GREEN}✓${NC} Certificates obtained"

# Copy certificates to nginx ssl directory
echo -e "${BLUE}[4/5] Copying certificates...${NC}"
sudo cp /etc/letsencrypt/live/qorsync.dev/fullchain.pem /etc/nginx/ssl/qorsync.dev/
sudo cp /etc/letsencrypt/live/qorsync.dev/privkey.pem /etc/nginx/ssl/qorsync.dev/
sudo cp /etc/letsencrypt/live/qorsync.dev/fullchain.pem /etc/nginx/ssl/qorsync.dev/
sudo cp /etc/letsencrypt/live/qorsync.dev/privkey.pem /etc/nginx/ssl/qorsync.dev/
sudo chmod 644 /etc/nginx/ssl/qorsync.dev/*.pem
sudo chmod 644 /etc/nginx/ssl/qorsync.dev/*.pem
echo -e "${GREEN}✓${NC} Certificates copied"

# Set up auto-renewal
echo -e "${BLUE}[5/5] Setting up auto-renewal...${NC}"
sudo systemctl enable certbot.timer
sudo systemctl start certbot.timer

# Create renewal hook script
sudo tee /etc/letsencrypt/renewal-hooks/deploy/nginx-reload.sh > /dev/null <<'EOF'
#!/bin/bash
cp /etc/letsencrypt/live/qorsync.dev/fullchain.pem /etc/nginx/ssl/qorsync.dev/
cp /etc/letsencrypt/live/qorsync.dev/privkey.pem /etc/nginx/ssl/qorsync.dev/
cp /etc/letsencrypt/live/qorsync.dev/fullchain.pem /etc/nginx/ssl/qorsync.dev/
cp /etc/letsencrypt/live/qorsync.dev/privkey.pem /etc/nginx/ssl/qorsync.dev/
docker compose -f /data/Demiurge-Blockchain/docker/docker-compose.production.yml restart nginx
EOF

sudo chmod +x /etc/letsencrypt/renewal-hooks/deploy/nginx-reload.sh
echo -e "${GREEN}✓${NC} Auto-renewal configured"

echo ""
echo "╔════════════════════════════════════════════════════════╗"
echo "║   SSL SETUP COMPLETE                                   ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""
echo -e "${GREEN}✓${NC} Certificates obtained and configured"
echo -e "${GREEN}✓${NC} Auto-renewal enabled"
echo ""
echo "Next steps:"
echo "  1. Start Nginx: docker compose -f docker/docker-compose.production.yml up -d nginx"
echo "  2. Test SSL: curl -I https://qorsync.dev"
echo "  3. Verify: https://www.ssllabs.com/ssltest/analyze.html?d=qorsync.dev"
echo ""
