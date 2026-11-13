#!/bin/bash
# Retired before any side effects; kept for source context only.
printf '%s\n' 'Retired deployment helper. Use docs/architecture/HOSTING.md and ADR-130; no deployment was performed.' >&2
exit 2

set -e

echo "🔒 SSL Setup for qorsync.dev"
echo "======================================"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "❌ Please run as root (use sudo)"
    exit 1
fi

# Check if certbot is installed
if ! command -v certbot &> /dev/null; then
    echo "📦 Installing certbot..."
    apt-get update
    apt-get install -y certbot python3-certbot-nginx
fi

# Create directories
echo "📁 Creating directories..."
mkdir -p /etc/nginx/ssl/qorsync.dev
mkdir -p /var/www/certbot
chown -R ubuntu:ubuntu /var/www/certbot 2>/dev/null || chown -R www-data:www-data /var/www/certbot

# Check if nginx is running
if systemctl is-active --quiet nginx; then
    echo "✅ Nginx is running"
else
    echo "⚠️  Nginx is not running. Starting nginx..."
    systemctl start nginx
fi

# Obtain certificate for qorsync.dev
echo ""
echo "🔐 Requesting SSL certificate for qorsync.dev..."
echo ""

# Use standalone mode (nginx will be stopped temporarily)
certbot certonly --standalone \
    --preferred-challenges http \
    -d qorsync.dev \
    --email [retired contact] \
    --agree-tos \
    --non-interactive || {
    echo "❌ Failed to obtain certificate for qorsync.dev"
    echo "   Make sure DNS is pointing to this server"
    exit 1
}

# Copy certificates to nginx ssl directory
echo ""
echo "📋 Copying certificates..."
cp /etc/letsencrypt/live/qorsync.dev/fullchain.pem /etc/nginx/ssl/qorsync.dev/
cp /etc/letsencrypt/live/qorsync.dev/privkey.pem /etc/nginx/ssl/qorsync.dev/
chmod 644 /etc/nginx/ssl/qorsync.dev/*.pem
chmod 600 /etc/nginx/ssl/qorsync.dev/privkey.pem

echo "✅ Certificates copied to /etc/nginx/ssl/qorsync.dev/"

# Set up auto-renewal
echo ""
echo "🔄 Setting up auto-renewal..."
systemctl enable certbot.timer 2>/dev/null || true
systemctl start certbot.timer 2>/dev/null || true

# Create renewal hook to copy certificates
cat > /etc/letsencrypt/renewal-hooks/deploy/nginx-reload.sh << 'EOF'
#!/bin/bash
# Copy certificates after renewal
cp /etc/letsencrypt/live/qorsync.dev/fullchain.pem /etc/nginx/ssl/qorsync.dev/
cp /etc/letsencrypt/live/qorsync.dev/privkey.pem /etc/nginx/ssl/qorsync.dev/
chmod 644 /etc/nginx/ssl/qorsync.dev/*.pem
chmod 600 /etc/nginx/ssl/qorsync.dev/privkey.pem
systemctl reload nginx
EOF

chmod +x /etc/letsencrypt/renewal-hooks/deploy/nginx-reload.sh

echo ""
echo "✅ SSL setup complete!"
echo ""
echo "📋 Next steps:"
echo "   1. Update nginx.conf with SSL paths (if not already done)"
echo "   2. Test nginx config: sudo nginx -t"
echo "   3. Reload nginx: sudo systemctl reload nginx"
echo "   4. Test HTTPS: curl https://rpc.qorsync.dev"
echo ""
echo "🔗 Certificate location:"
echo "   /etc/letsencrypt/live/qorsync.dev/"
echo "   /etc/nginx/ssl/qorsync.dev/"
echo ""
