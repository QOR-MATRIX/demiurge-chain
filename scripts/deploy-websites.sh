#!/bin/bash
# Retired before any side effects; kept for source context only.
printf '%s\n' 'Retired deployment helper. Use docs/architecture/HOSTING.md and ADR-130; no deployment was performed.' >&2
exit 2

# Website Deployment Script
# Deploys qorsync.dev and qorsync.dev to production

set -e

echo "🚀 Demiurge Website Deployment"
echo "========================================"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
DEPLOY_HOST="pleroma"
DEPLOY_USER="ubuntu"
DEPLOY_DIR="/home/ubuntu/demiurge"
REPO_DIR="X:\Demiurge-Blockchain"

# Step 1: Build websites locally
echo -e "${BLUE}[1/6] Building websites...${NC}"

echo "  Building marketing site..."
cd "$REPO_DIR/apps/marketing-site"
npm install --production
npm run build

echo "  Building AI website..."
cd "$REPO_DIR/ai-website"
npm install --production
npm run build

echo -e "${GREEN}✅ Websites built${NC}"

# Step 2: Create Docker images
echo ""
echo -e "${BLUE}[2/6] Creating Docker images...${NC}"

echo "  Building marketing site image..."
docker build -f "$REPO_DIR/apps/marketing-site/Dockerfile" \
  -t localhost:5000/demiurge-marketing:latest \
  "$REPO_DIR/apps/marketing-site"

echo "  Building AI website image..."
docker build -f "$REPO_DIR/ai-website/Dockerfile" \
  -t localhost:5000/demiurge-ai-website:latest \
  "$REPO_DIR/ai-website"

echo -e "${GREEN}✅ Docker images created${NC}"

# Step 3: Push to server
echo ""
echo -e "${BLUE}[3/6] Pushing to production server...${NC}"

# Push docker-compose and nginx config
scp "$REPO_DIR/docker/docker-compose.websites.yml" \
  "$DEPLOY_USER@$DEPLOY_HOST:$DEPLOY_DIR/"

scp "$REPO_DIR/docker/nginx.websites.conf" \
  "$DEPLOY_USER@$DEPLOY_HOST:$DEPLOY_DIR/"

# Push environment files
scp "$REPO_DIR/apps/marketing-site/.env.production" \
  "$DEPLOY_USER@$DEPLOY_HOST:$DEPLOY_DIR/marketing.env"

scp "$REPO_DIR/ai-website/.env.production" \
  "$DEPLOY_USER@$DEPLOY_HOST:$DEPLOY_DIR/ai-website.env"

echo -e "${GREEN}✅ Files deployed to server${NC}"

# Step 4: Deploy on server
echo ""
echo -e "${BLUE}[4/6] Deploying on server...${NC}"

ssh "$DEPLOY_USER@$DEPLOY_HOST" << 'EOF'
  cd ~/demiurge
  
  # Stop old containers
  docker compose -f docker-compose.websites.yml down 2>/dev/null || true
  
  # Start new containers
  docker compose -f docker-compose.websites.yml up -d
  
  # Wait for services
  sleep 10
  
  # Check health
  echo "Checking service health..."
  curl -s -f http://localhost:3000/health || echo "AI website not yet ready"
  curl -s -f http://localhost:3001/health || echo "Marketing site not yet ready"
EOF

echo -e "${GREEN}✅ Services deployed${NC}"

# Step 5: Verify SSL
echo ""
echo -e "${BLUE}[5/6] Verifying SSL certificates...${NC}"

echo "  Testing qorsync.dev..."
curl -sI https://qorsync.dev | head -3

echo ""
echo "  Testing qorsync.dev..."
curl -sI https://qorsync.dev | head -3

echo -e "${GREEN}✅ SSL certificates verified${NC}"

# Step 6: Health checks
echo ""
echo -e "${BLUE}[6/6] Running health checks...${NC}"

echo "  Checking RPC integration..."
curl -s -X POST https://rpc.qorsync.dev \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"system_chain","params":[],"id":1}' | jq .

echo ""
echo -e "${GREEN}✅ Health checks passed${NC}"

echo ""
echo "========================================"
echo -e "${GREEN}🎉 DEPLOYMENT COMPLETE${NC}"
echo "========================================"
echo ""
echo "Endpoints:"
echo "  - https://qorsync.dev (AI Codex)"
echo "  - https://qorsync.dev (Marketing)"
echo "  - https://rpc.qorsync.dev (Blockchain RPC)"
echo ""
echo "Logs:"
echo "  docker logs demiurge-ai-website"
echo "  docker logs demiurge-marketing-site"
echo "  docker logs demiurge-nginx"
echo ""
