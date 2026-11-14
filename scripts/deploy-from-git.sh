#!/bin/bash
# Retired before any side effects; kept for source context only.
printf '%s\n' 'Retired deployment helper. Use docs/architecture/HOSTING.md and ADR-130; no deployment was performed.' >&2
exit 2

# Deploy Blockchain Connection Fix from Git
# This script pulls latest changes and deploys them
# Run on server: sudo bash /tmp/deploy-from-git.sh

set -e

echo "🚀 Deploying Blockchain Connection Fix from Git..."
echo ""

# Navigate to project directory
cd /data/Demiurge-Blockchain || { echo "❌ Project directory not found"; exit 1; }

# Backup current nginx.conf
echo "📦 Backing up current nginx.conf..."
if [ -f docker/nginx.conf ]; then
    cp docker/nginx.conf docker/nginx.conf.backup.$(date +%Y%m%d_%H%M%S)
fi

# Pull latest changes from git
echo "📥 Pulling latest changes from git..."
git pull origin main || {
    echo "⚠️  Git pull failed. Continuing with existing files..."
}

# Verify critical files exist
if [ ! -f docker/nginx.conf ]; then
    echo "❌ docker/nginx.conf not found after git pull!"
    exit 1
fi

if [ ! -f apps/hub/src/lib/blockchain.ts ]; then
    echo "❌ apps/hub/src/lib/blockchain.ts not found after git pull!"
    exit 1
fi

# Test Nginx configuration
echo "🧪 Testing Nginx configuration..."
docker compose -f docker/docker-compose.production.yml exec nginx nginx -t || {
    echo "❌ Nginx configuration test failed!"
    echo "   Restoring backup..."
    if ls docker/nginx.conf.backup.* 1> /dev/null 2>&1; then
        cp docker/nginx.conf.backup.* docker/nginx.conf 2>/dev/null || true
    fi
    exit 1
}

# Restart Nginx
echo "🔄 Restarting Nginx..."
docker compose -f docker/docker-compose.production.yml restart nginx

# Check if blockchain node is running
echo "🔍 Checking blockchain node status..."
if docker ps --filter 'name=demiurge-node' --format '{{.Names}}' | grep -q demiurge-node; then
    echo "✅ Blockchain node is running"
    docker logs demiurge-node --tail 5
else
    echo "⚠️  Blockchain node is not running. Starting it..."
    docker compose -f docker/docker-compose.production.yml up -d demiurge-node
    echo "⏳ Waiting for node to start..."
    sleep 10
    docker logs demiurge-node --tail 20
fi

# Rebuild Hub with updated blockchain client
echo "🔨 Rebuilding Hub container..."
docker compose -f docker/docker-compose.production.yml build hub

# Restart Hub
echo "🔄 Restarting Hub..."
docker compose -f docker/docker-compose.production.yml up -d hub

# Wait for services to be ready
echo "⏳ Waiting for services to be ready..."
sleep 10

# Check service status
echo ""
echo "📊 Service Status:"
docker compose -f docker/docker-compose.production.yml ps

echo ""
echo "✅ Deployment complete!"
echo ""
echo "🧪 Testing:"
echo "   1. Visit https://qorsync.dev"
echo "   2. Check browser console for [Blockchain] logs"
echo "   3. Verify blockchain status banner disappears"
echo ""
echo "📋 Logs:"
echo "   Nginx: docker logs demiurge-nginx --tail 50"
echo "   Hub: docker logs demiurge-hub --tail 50"
echo "   Node: docker logs demiurge-node --tail 50"
echo ""
echo "🔍 Test WebSocket connection:"
echo "   curl -i -N -H 'Connection: Upgrade' -H 'Upgrade: websocket' -H 'Sec-WebSocket-Version: 13' -H 'Sec-WebSocket-Key: test' https://qorsync.dev/rpc"
