# Idempotent Proxy Setup Guide

## Overview

This guide explains how to set up the idempotent proxy server from [ldclabs/idempotent-proxy](https://github.com/ldclabs/idempotent-proxy) with Docker and Nginx for production use.

## Prerequisites

- Linux server (Ubuntu/Debian)
- Docker
- Docker Compose
- Nginx
- SSL certificates

## Installation Steps

1. Install Docker Compose

```bash
# Download Docker Compose binary
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose

# Make it executable
sudo chmod +x /usr/local/bin/docker-compose
```

2. Create Project Structure

```bash
mkdir -p ~/projects/idempotent-proxy
cd ~/projects/idempotent-proxy
```

3. Configure Docker Compose

```yaml
services:
  idempotent-proxy:
    build: .
    container_name: idempotent-proxy
    ports:
      - "8080:8080"
    restart: always
    environment:
      - SERVER_ADDR=0.0.0.0:8080
    volumes:
      - type: bind
        source: /root/projects/idempotent-proxy/.env
        target: /app/.env
        read_only: true
```

4. Configure Nginx

```nginx
server {
    listen 80;
    server_name ic2p2ramp.xyz;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name ic2p2ramp.xyz;

    ssl_certificate /etc/letsencrypt/live/ic2p2ramp.xyz/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/ic2p2ramp.xyz/privkey.pem;

    location / {
        resolver 127.0.0.11 valid=30s;
        proxy_pass http://localhost:8080;

        proxy_set_header Host $http_x_forwarded_host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        proxy_connect_timeout 60s;
        proxy_read_timeout 60s;
        proxy_buffering on;
        proxy_buffer_size 16k;
        proxy_buffers 4 16k;

        access_log /var/log/nginx/proxy_access.log;
        error_log /var/log/nginx/proxy_error.log;
    }
}
```

5. Create Systemd Service

```systemd
[Unit]
Description=Idempotent Proxy Docker Compose
Requires=docker.service
After=docker.service

[Service]
Type=simple
User=root
WorkingDirectory=/root/projects/idempotent-proxy
ExecStartPre=-/usr/local/bin/docker-compose down
ExecStart=/usr/local/bin/docker-compose up
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

6. Enable and Start Services

```bash
# Enable and start Nginx
sudo ln -s /etc/nginx/sites-available/proxy /etc/nginx/sites-enabled/
sudo systemctl restart nginx

# Enable and start idempotent proxy
sudo systemctl daemon-reload
sudo systemctl enable idempotent-proxy
sudo systemctl start idempotent-proxy
```

## Usage Example

### PayPal API Request

```bash
# Encode credentials
export PAYPAL_CLIENT_ID="your_client_id"
export PAYPAL_CLIENT_SECRET="your_client_secret"
export CREDENTIALS=$(echo -n "${PAYPAL_CLIENT_ID}:${PAYPAL_CLIENT_SECRET}" | base64 | tr -d '\n')

# Make API request
curl -X POST "https://ic2p2ramp.xyz/v1/oauth2/token" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -H "Authorization: Basic ${CREDENTIALS}" \
    -H "x-forwarded-host: api-m.sandbox.paypal.com" \
    -H "idempotency-key: auth-key-0" \
    -d "grant_type=client_credentials"
```

## Useful Commands

### Service Management

```bash
# View service status
sudo systemctl status idempotent-proxy

# View logs
sudo journalctl -u idempotent-proxy -f

# Restart service
sudo systemctl restart idempotent-proxy
```

### Docker Management

```bash
# View container logs
docker logs -f idempotent-proxy

# Rebuild and restart container
cd ~/projects/idempotent-proxy
docker-compose build --no-cache
docker-compose up -d
```

### Troubleshooting

- Check service logs: `journalctl -u idempotent-proxy -f`
- Check Nginx logs: `tail -f /var/log/nginx/proxy_error.log`
- Verify container status: `docker ps`
- Check container logs: `docker logs idempotent-proxy`
