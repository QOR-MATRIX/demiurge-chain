#!/bin/bash
# Retired before any side effects; kept for source context only.
printf '%s\n' 'Retired deployment helper. Use docs/architecture/HOSTING.md and ADR-130; no deployment was performed.' >&2
exit 2

set -e

echo "╔════════════════════════════════════════════════════════╗"
echo "║   WIPE AND REDEPLOY - DEMIURGE BLOCKCHAIN             ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}❌ Please run as root (use sudo)${NC}"
    exit 1
fi

DEPLOY_DIR="/opt/demiurge-blockchain"
DOCKER_DIR="$DEPLOY_DIR/docker"

echo -e "${YELLOW}⚠️  WARNING: This will wipe all containers, volumes, and data!${NC}"
echo -e "${YELLOW}⚠️  Only proceed if you have backups or are okay with data loss.${NC}"
echo ""

# Check for non-interactive mode
if [ "$1" == "--yes" ] || [ "$1" == "-y" ]; then
    confirm="WIPE"
    echo "Non-interactive mode: proceeding with wipe..."
else
    read -p "Type 'WIPE' to confirm: " confirm
fi

if [ "$confirm" != "WIPE" ]; then
    echo -e "${RED}❌ Aborted.${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}[1/8] Stopping all services...${NC}"
cd "$DOCKER_DIR" || exit 1
docker compose -f docker-compose.production.yml down 2>/dev/null || true
docker stop $(docker ps -aq) 2>/dev/null || true
echo -e "${GREEN}✓${NC} Services stopped"

echo ""
echo -e "${BLUE}[2/8] Removing all containers...${NC}"
docker rm -f $(docker ps -aq) 2>/dev/null || true
echo -e "${GREEN}✓${NC} Containers removed"

echo ""
echo -e "${BLUE}[3/8] Removing all volumes...${NC}"
docker volume rm $(docker volume ls -q) 2>/dev/null || true
echo -e "${GREEN}✓${NC} Volumes removed"

echo ""
echo -e "${BLUE}[4/8] Removing all networks...${NC}"
docker network prune -f 2>/dev/null || true
echo -e "${GREEN}✓${NC} Networks cleaned"

echo ""
echo -e "${BLUE}[5/8] Pulling latest code...${NC}"
cd "$DEPLOY_DIR" || exit 1
git fetch origin
git reset --hard origin/main
git clean -fd
echo -e "${GREEN}✓${NC} Code updated"

echo ""
echo -e "${BLUE}[6/8] Building Docker images...${NC}"
cd "$DOCKER_DIR" || exit 1
docker compose -f docker-compose.production.yml build --no-cache
echo -e "${GREEN}✓${NC} Images built"

echo ""
echo -e "${BLUE}[7/8] Starting services...${NC}"
docker compose -f docker-compose.production.yml up -d
echo -e "${GREEN}✓${NC} Services started"

echo ""
echo -e "${BLUE}[8/8] Waiting for services to be healthy...${NC}"
sleep 10

# Wait for postgres
echo "Waiting for PostgreSQL..."
for i in {1..30}; do
    if docker exec demiurge-postgres pg_isready -U qor_auth -d qor_auth >/dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} PostgreSQL is ready"
        break
    fi
    sleep 2
done

# Wait for redis
echo "Waiting for Redis..."
for i in {1..30}; do
    if docker exec demiurge-redis redis-cli ping >/dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} Redis is ready"
        break
    fi
    sleep 2
done

# Wait for qor-auth
echo "Waiting for QOR Auth..."
for i in {1..60}; do
    if docker exec demiurge-qor-auth curl -f http://localhost:8080/health >/dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} QOR Auth is ready"
        break
    fi
    sleep 2
done

# Wait for hub
echo "Waiting for Hub..."
for i in {1..60}; do
    if curl -f http://localhost:3000 >/dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} Hub is ready"
        break
    fi
    sleep 2
done

echo ""
echo -e "${BLUE}[9/9] Setting up SSL certificates...${NC}"
if [ ! -f /etc/letsencrypt/live/qorsync.dev/fullchain.pem ]; then
    echo "Obtaining SSL certificate for qorsync.dev..."
    docker stop demiurge-nginx 2>/dev/null || true
    certbot certonly --standalone \
        -d qorsync.dev \
        --email [retired contact] \
        --agree-tos \
        --non-interactive || {
        echo -e "${YELLOW}⚠${NC}  SSL certificate setup failed (may need DNS check)"
    }
    docker start demiurge-nginx 2>/dev/null || true
fi

# Copy certificates if they exist
if [ -f /etc/letsencrypt/live/qorsync.dev/fullchain.pem ]; then
    mkdir -p "$DOCKER_DIR/ssl/qorsync.dev"
    cp /etc/letsencrypt/live/qorsync.dev/fullchain.pem "$DOCKER_DIR/ssl/qorsync.dev/"
    cp /etc/letsencrypt/live/qorsync.dev/privkey.pem "$DOCKER_DIR/ssl/qorsync.dev/"
    chmod 644 "$DOCKER_DIR/ssl/qorsync.dev"/*.pem
    chmod 600 "$DOCKER_DIR/ssl/qorsync.dev/privkey.pem"
    echo -e "${GREEN}✓${NC} SSL certificates copied"
fi

# Restart nginx to pick up SSL
docker restart demiurge-nginx 2>/dev/null || true

echo ""
echo "╔════════════════════════════════════════════════════════╗"
echo "║   REDEPLOYMENT COMPLETE                                ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""
echo -e "${GREEN}✓${NC} All services redeployed"
echo ""
echo "Service Status:"
docker compose -f "$DOCKER_DIR/docker-compose.production.yml" ps
echo ""
echo "Next steps:"
echo "  1. Check service logs: docker compose -f $DOCKER_DIR/docker-compose.production.yml logs -f"
echo "  2. Test endpoints:"
echo "     - https://qorsync.dev"
echo "     - https://rpc.qorsync.dev"
echo "  3. Verify SSL: curl -I https://rpc.qorsync.dev"
echo ""
