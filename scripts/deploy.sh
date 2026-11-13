#!/bin/bash
# Retired before any side effects; kept for source context only.
printf '%s\n' 'Retired deployment helper. Use docs/architecture/HOSTING.md and ADR-130; no deployment was performed.' >&2
exit 2

# =============================================================================
# Demiurge-Cloud - Production Deployment Script
# =============================================================================
# Usage: ./scripts/deploy.sh [component]
# Components: all, node, hub, auth, nginx, secrets
# =============================================================================

set -e  # Exit on error

# Configuration
DEPLOY_USER="ubuntu"
DEPLOY_HOST="51.210.209.112"
DEPLOY_PATH="/data/Demiurge-Blockchain"
SSH_KEY=".ssh/id_ed25519_pleroma"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${CYAN}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# SSH wrapper
remote_exec() {
    ssh -o BatchMode=yes -o ConnectTimeout=10 -o StrictHostKeyChecking=no \
        -i "$SSH_KEY" "${DEPLOY_USER}@${DEPLOY_HOST}" "$1"
}

# SCP wrapper
remote_copy() {
    scp -o StrictHostKeyChecking=no -i "$SSH_KEY" "$1" "${DEPLOY_USER}@${DEPLOY_HOST}:$2"
}

# Generate secure secrets
generate_secrets() {
    log_info "Generating cryptographically secure secrets..."
    
    JWT_ACCESS=$(openssl rand -base64 64 | tr -d '\n')
    JWT_REFRESH=$(openssl rand -base64 64 | tr -d '\n')
    DB_PASSWORD=$(openssl rand -base64 32 | tr -d '\n' | tr -dc 'a-zA-Z0-9')
    
    log_info "Creating production secrets file..."
    cat > config/production/.secrets << EOF
# Demiurge Production Secrets
# Generated: $(date -Iseconds)
# SECURITY: Never commit this file to git!

JWT_ACCESS_SECRET=${JWT_ACCESS}
JWT_REFRESH_SECRET=${JWT_REFRESH}
DATABASE_PASSWORD=${DB_PASSWORD}
EOF
    
    chmod 600 config/production/.secrets
    log_success "Secrets generated. Upload manually with: deploy.sh secrets"
}

# Deploy secrets to server
deploy_secrets() {
    log_info "Deploying secrets to server..."
    
    if [ ! -f "config/production/.secrets" ]; then
        log_error "No secrets file found. Run: ./deploy.sh generate-secrets"
    fi
    
    remote_copy "config/production/.secrets" "${DEPLOY_PATH}/config/production/.secrets"
    remote_exec "chmod 600 ${DEPLOY_PATH}/config/production/.secrets"
    log_success "Secrets deployed"
}

# Deploy blockchain node
deploy_node() {
    log_info "Deploying Demiurge Node..."
    
    # Sync framework code
    log_info "Syncing framework code..."
    rsync -avz --delete --exclude='target' \
        -e "ssh -i $SSH_KEY" \
        framework/ "${DEPLOY_USER}@${DEPLOY_HOST}:${DEPLOY_PATH}/framework/"
    
    # Build on server
    log_info "Building blockchain node..."
    remote_exec "source ~/.cargo/env && cd ${DEPLOY_PATH}/framework && cargo build --release"
    
    # Install and restart service
    log_info "Restarting node service..."
    remote_copy "config/production/demiurge-node.service" "/tmp/demiurge-node.service"
    remote_exec "sudo mv /tmp/demiurge-node.service /etc/systemd/system/ && \
                 sudo systemctl daemon-reload && \
                 sudo systemctl enable demiurge-node && \
                 sudo systemctl restart demiurge-node"
    
    sleep 3
    remote_exec "sudo systemctl status demiurge-node --no-pager | head -10"
    log_success "Node deployed"
}

# Deploy Hub frontend
deploy_hub() {
    log_info "Deploying Demiurge Hub..."
    
    # Sync app code
    log_info "Syncing hub code..."
    rsync -avz --delete \
        --exclude='node_modules' --exclude='.next' --exclude='.env.local' \
        -e "ssh -i $SSH_KEY" \
        apps/hub/ "${DEPLOY_USER}@${DEPLOY_HOST}:${DEPLOY_PATH}/apps/hub/"
    
    # Sync packages
    rsync -avz --delete --exclude='node_modules' \
        -e "ssh -i $SSH_KEY" \
        packages/ "${DEPLOY_USER}@${DEPLOY_HOST}:${DEPLOY_PATH}/packages/"
    
    # Build on server
    log_info "Installing dependencies and building..."
    remote_exec "cd ${DEPLOY_PATH} && npm install && \
                 npm run build --workspace=@demiurge/qor-sdk && \
                 npm run build --workspace=@demiurge/ui-shared && \
                 npm run build --workspace=@demiurge/hub"
    
    # Copy env file
    remote_copy "config/production/.env.hub" "${DEPLOY_PATH}/apps/hub/.env.production"
    
    # Restart with PM2 (or systemd if preferred)
    log_info "Restarting Hub..."
    remote_exec "pm2 restart hub || pm2 start npm --name hub --cwd ${DEPLOY_PATH}/apps/hub -- start"
    
    log_success "Hub deployed"
}

# Deploy QOR Auth
deploy_auth() {
    log_info "Deploying QOR Auth..."
    
    # Sync service code
    log_info "Syncing auth service code..."
    rsync -avz --delete --exclude='target' \
        -e "ssh -i $SSH_KEY" \
        services/qor-auth/ "${DEPLOY_USER}@${DEPLOY_HOST}:${DEPLOY_PATH}/services/qor-auth/"
    
    # Build on server
    log_info "Building QOR Auth..."
    remote_exec "source ~/.cargo/env && cd ${DEPLOY_PATH}/services/qor-auth && cargo build --release"
    
    # Copy config
    remote_copy "config/production/qor-auth.toml" "${DEPLOY_PATH}/services/qor-auth/config/production.toml"
    
    # Restart with PM2
    log_info "Restarting QOR Auth..."
    remote_exec "pm2 restart qor-auth || \
                 cd ${DEPLOY_PATH}/services/qor-auth && \
                 RUN_ENV=production pm2 start ./target/release/qor-auth --name qor-auth"
    
    log_success "QOR Auth deployed"
}

# Deploy Nginx config
deploy_nginx() {
    log_info "Deploying Nginx configuration..."
    
    remote_copy "config/production/nginx.conf" "/tmp/nginx.conf"
    remote_exec "sudo mv /tmp/nginx.conf /etc/nginx/nginx.conf && \
                 sudo nginx -t && \
                 sudo systemctl reload nginx"
    
    log_success "Nginx deployed"
}

# Health check
health_check() {
    log_info "Running health checks..."
    
    echo ""
    log_info "Checking services..."
    remote_exec "pm2 list 2>/dev/null || echo 'PM2 not running'"
    remote_exec "sudo systemctl status demiurge-node --no-pager | head -5"
    
    echo ""
    log_info "Checking endpoints..."
    echo "Hub: $(curl -s -o /dev/null -w '%{http_code}' https://qorsync.dev)"
    echo "Auth: $(curl -s -o /dev/null -w '%{http_code}' https://qorsync.dev/health)"
    echo "RPC: $(curl -s -o /dev/null -w '%{http_code}' https://rpc.qorsync.dev)"
    
    log_success "Health check complete"
}

# Full deployment
deploy_all() {
    log_info "Starting full deployment..."
    deploy_nginx
    deploy_node
    deploy_auth
    deploy_hub
    health_check
    log_success "Full deployment complete!"
}

# Main
case "${1:-help}" in
    all)
        deploy_all
        ;;
    node)
        deploy_node
        ;;
    hub)
        deploy_hub
        ;;
    auth)
        deploy_auth
        ;;
    nginx)
        deploy_nginx
        ;;
    secrets)
        deploy_secrets
        ;;
    generate-secrets)
        generate_secrets
        ;;
    health)
        health_check
        ;;
    *)
        echo "Demiurge Deployment Script"
        echo ""
        echo "Usage: $0 <command>"
        echo ""
        echo "Commands:"
        echo "  all              Deploy everything"
        echo "  node             Deploy blockchain node"
        echo "  hub              Deploy Hub frontend"
        echo "  auth             Deploy QOR Auth service"
        echo "  nginx            Deploy Nginx config"
        echo "  secrets          Upload secrets to server"
        echo "  generate-secrets Generate new secure secrets"
        echo "  health           Run health checks"
        ;;
esac
