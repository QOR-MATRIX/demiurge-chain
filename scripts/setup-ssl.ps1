
# Retired before any side effects; kept for source context only.
throw "Retired deployment helper. Use docs/architecture/HOSTING.md and ADR-130; no deployment was performed."

# SSL/TLS Certificate Setup
# Configures Let's Encrypt SSL certificates for production

$ErrorActionPreference = "Stop"

Write-Host "🔒 SSL/TLS Certificate Setup" -ForegroundColor Cyan
Write-Host ""

# Configuration
$SERVER = "51.210.209.112"
$DOMAIN = "qorsync.dev" # Update with your domain
$EMAIL = "[retired contact]" # Update with your email

Write-Host "📋 Prerequisites" -ForegroundColor Yellow
Write-Host ""
Write-Host "1. Domain name pointing to $SERVER" -ForegroundColor Gray
Write-Host "2. Ports 80 and 443 open in firewall" -ForegroundColor Gray
Write-Host "3. Nginx installed and configured" -ForegroundColor Gray
Write-Host ""

$confirm = Read-Host "Do you have a domain name configured? (yes/no)"
if ($confirm -ne "yes") {
    Write-Host "⚠️  SSL setup requires a domain name." -ForegroundColor Yellow
    Write-Host "   Update DNS A record for your domain to point to $SERVER" -ForegroundColor Gray
    exit 0
}

Write-Host ""
Write-Host "🔧 Setup Steps" -ForegroundColor Yellow
Write-Host ""

# Step 1: Install Certbot
Write-Host "1. Install Certbot:" -ForegroundColor Cyan
Write-Host ""
Write-Host "   SSH into server:" -ForegroundColor Gray
Write-Host "   ssh $SERVER" -ForegroundColor DarkGray
Write-Host ""
Write-Host "   Install certbot:" -ForegroundColor Gray
Write-Host "   sudo apt update" -ForegroundColor DarkGray
Write-Host "   sudo apt install certbot python3-certbot-nginx" -ForegroundColor DarkGray
Write-Host ""

# Step 2: Configure Nginx
Write-Host "2. Configure Nginx:" -ForegroundColor Cyan
Write-Host ""
Write-Host "   Create /etc/nginx/sites-available/demiurge:" -ForegroundColor Gray
Write-Host @"
server {
    listen 80;
    server_name $DOMAIN;

    # QOR Auth API
    location /api/auth {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade `$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host `$host;
        proxy_cache_bypass `$http_upgrade;
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto `$scheme;
    }

    # Next.js Hub
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade `$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host `$host;
        proxy_cache_bypass `$http_upgrade;
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto `$scheme;
    }

    # Static assets caching
    location /_next/static {
        proxy_pass http://127.0.0.1:3000;
        proxy_cache_valid 200 365d;
        add_header Cache-Control "public, immutable";
    }
}
"@ -ForegroundColor DarkGray
Write-Host ""
Write-Host "   Enable site:" -ForegroundColor Gray
Write-Host "   sudo ln -s /etc/nginx/sites-available/demiurge /etc/nginx/sites-enabled/" -ForegroundColor DarkGray
Write-Host "   sudo nginx -t" -ForegroundColor DarkGray
Write-Host "   sudo systemctl reload nginx" -ForegroundColor DarkGray
Write-Host ""

# Step 3: Obtain Certificate
Write-Host "3. Obtain SSL Certificate:" -ForegroundColor Cyan
Write-Host ""
Write-Host "   Run certbot:" -ForegroundColor Gray
Write-Host "   sudo certbot --nginx -d $DOMAIN --email $EMAIL --agree-tos --non-interactive" -ForegroundColor DarkGray
Write-Host ""
Write-Host "   Or interactive mode:" -ForegroundColor Gray
Write-Host "   sudo certbot --nginx -d $DOMAIN" -ForegroundColor DarkGray
Write-Host ""

# Step 4: Auto-renewal
Write-Host "4. Set Up Auto-Renewal:" -ForegroundColor Cyan
Write-Host ""
Write-Host "   Certbot automatically sets up renewal." -ForegroundColor Gray
Write-Host "   Test renewal:" -ForegroundColor Gray
Write-Host "   sudo certbot renew --dry-run" -ForegroundColor DarkGray
Write-Host ""

# Step 5: Update Environment Variables
Write-Host "5. Update Environment Variables:" -ForegroundColor Cyan
Write-Host ""
Write-Host "   Update apps/hub/.env.production:" -ForegroundColor Gray
Write-Host "   NEXT_PUBLIC_QOR_AUTH_URL=https://$DOMAIN/api/auth" -ForegroundColor DarkGray
Write-Host "   NEXT_PUBLIC_BLOCKCHAIN_WS_URL=wss://$DOMAIN/ws" -ForegroundColor DarkGray
Write-Host ""

Write-Host "✅ SSL setup instructions provided!" -ForegroundColor Green
Write-Host ""
Write-Host "📝 After setup, verify SSL:" -ForegroundColor Yellow
Write-Host "   curl -I https://$DOMAIN" -ForegroundColor DarkGray
Write-Host "   openssl s_client -connect $DOMAIN:443" -ForegroundColor DarkGray
