#!/bin/bash
# Retired before any side effects; kept for source context only.
printf '%s\n' 'Retired deployment helper. Use docs/architecture/HOSTING.md and ADR-130; no deployment was performed.' >&2
exit 2

set -e

echo "SSL Setup for qorsync.dev (qorsync.dev will be configured after DNS)"
echo ""

# Create directories
sudo mkdir -p /etc/nginx/ssl/qorsync.dev
sudo mkdir -p /var/www/certbot
sudo chown -R ubuntu:ubuntu /var/www/certbot

# Stop nginx temporarily
docker compose -f /data/Demiurge-Blockchain/docker/docker-compose.production.yml stop nginx 2>/dev/null || true

# Obtain certificate for qorsync.dev
echo "Requesting certificate for qorsync.dev..."
sudo certbot certonly --standalone \
    --preferred-challenges http \
    -d qorsync.dev \
    -d qorsync.dev \
    --email [retired contact] \
    --agree-tos \
    --non-interactive || {
    echo "Failed to obtain certificate for qorsync.dev"
    exit 1
}

# Copy certificates
sudo cp /etc/letsencrypt/live/qorsync.dev/fullchain.pem /etc/nginx/ssl/qorsync.dev/
sudo cp /etc/letsencrypt/live/qorsync.dev/privkey.pem /etc/nginx/ssl/qorsync.dev/
sudo chmod 644 /etc/nginx/ssl/qorsync.dev/*.pem

# Set up auto-renewal
sudo systemctl enable certbot.timer 2>/dev/null || true
sudo systemctl start certbot.timer 2>/dev/null || true

echo ""
echo "✓ Certificate for qorsync.dev obtained!"
echo "Note: qorsync.dev will be configured once DNS is set up"
echo ""
