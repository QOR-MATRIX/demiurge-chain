#!/bin/bash
# Retired before any side effects; kept for source context only.
printf '%s\n' 'Retired deployment helper. Use docs/architecture/HOSTING.md and ADR-130; no deployment was performed.' >&2
exit 2

# =============================================================================
# Demiurge Email Setup Script (Bash)
# =============================================================================
# This script helps configure SMTP credentials for email verification
# Run: ./scripts/setup-email.sh re_your_api_key
# =============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo ""
echo -e "${CYAN}========================================"
echo -e "  Demiurge Email Configuration Setup"
echo -e "========================================${NC}"
echo ""

# Get Resend API Key
if [ -z "$1" ]; then
    echo -e "${YELLOW}Enter your Resend API Key (starts with re_):${NC}"
    read -s RESEND_API_KEY
else
    RESEND_API_KEY="$1"
fi

if [ -z "$RESEND_API_KEY" ] || [[ ! "$RESEND_API_KEY" == re_* ]]; then
    echo -e "${RED}ERROR: Invalid API key. Resend API keys start with 're_'${NC}"
    exit 1
fi

# Optional: custom from email
FROM_EMAIL="${2:[retired contact]}"
FROM_NAME="${3:-Demiurge-Cloud}"

# Create secrets file
SECRETS_PATH="config/production/.secrets"
mkdir -p "$(dirname "$SECRETS_PATH")"

cat > "$SECRETS_PATH" << EOF
# Demiurge Production Secrets
# Generated: $(date -Iseconds)
# SECURITY: Never commit this file to git!

# SMTP Configuration (Resend)
SMTP_HOST=smtp.resend.com
SMTP_PORT=587
SMTP_USERNAME=resend
SMTP_PASSWORD=$RESEND_API_KEY
SMTP_FROM_EMAIL=$FROM_EMAIL
SMTP_FROM_NAME=$FROM_NAME
BASE_URL=https://qorsync.dev
EOF

chmod 600 "$SECRETS_PATH"
echo -e "${GREEN}Created local secrets file: $SECRETS_PATH${NC}"

echo ""
echo -e "${CYAN}========================================"
echo -e "  Next Steps - Deploy to Server"
echo -e "========================================${NC}"
echo ""
echo -e "${YELLOW}Option 1: Quick deploy (if SSH keys are set up):${NC}"
echo "  scp $SECRETS_PATH ubuntu@51.210.209.112:/data/Demiurge-Blockchain/config/production/"
echo ""
echo -e "${YELLOW}Option 2: Manual (copy the file via your preferred method)${NC}"
echo ""
echo -e "${YELLOW}Then on the server, run:${NC}"
echo "  cd /data/Demiurge-Blockchain"
echo "  git pull origin main"
echo "  cd services/qor-auth"
echo "  cargo build --release"
echo "  sudo systemctl restart qor-auth"
echo "  sudo journalctl -u qor-auth -f"
echo ""
echo -e "${GREEN}The logs should show: 'Email service configured'${NC}"
echo ""
