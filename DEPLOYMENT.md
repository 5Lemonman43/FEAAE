# Deployment Guide

This guide covers various deployment options for the Polymarket Arbitrage Bot.

## Prerequisites

- Rust 1.70+ installed
- Polymarket API credentials (optional for simulation)
- Linux, macOS, or Windows with WSL2
- Stable internet connection

## Local Deployment

### 1. Clone and Build

```bash
git clone <repository-url>
cd polymarket-arbitrage-bot
cargo build --release
```

### 2. Configure Environment

```bash
cp .env.example .env
nano .env  # Edit with your settings
```

### 3. Run

```bash
./target/release/polymarket-arbitrage-bot
```

## Production Deployment

### Option 1: Systemd Service (Linux)

Create a systemd service file:

```bash
sudo nano /etc/systemd/system/polymarket-bot.service
```

```ini
[Unit]
Description=Polymarket Arbitrage Bot
After=network.target

[Service]
Type=simple
User=your-user
WorkingDirectory=/path/to/polymarket-arbitrage-bot
EnvironmentFile=/path/to/polymarket-arbitrage-bot/.env
ExecStart=/path/to/polymarket-arbitrage-bot/target/release/polymarket-arbitrage-bot
Restart=always
RestartSec=10
StandardOutput=append:/var/log/polymarket-bot/output.log
StandardError=append:/var/log/polymarket-bot/error.log

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable polymarket-bot
sudo systemctl start polymarket-bot
sudo systemctl status polymarket-bot
```

View logs:

```bash
sudo journalctl -u polymarket-bot -f
```

### Option 2: Docker Container

Create `Dockerfile`:

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/polymarket-arbitrage-bot /usr/local/bin/

ENV RUST_LOG=info

CMD ["polymarket-arbitrage-bot"]
```

Build and run:

```bash
docker build -t polymarket-bot .

docker run -d \
  --name polymarket-bot \
  --restart unless-stopped \
  --env-file .env \
  polymarket-bot
```

View logs:

```bash
docker logs -f polymarket-bot
```

### Option 3: Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  bot:
    build: .
    container_name: polymarket-bot
    restart: unless-stopped
    env_file:
      - .env
    volumes:
      - ./logs:/var/log/polymarket
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

Run:

```bash
docker-compose up -d
docker-compose logs -f
```

### Option 4: Kubernetes Deployment

Create `k8s-deployment.yaml`:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: polymarket-bot-config
type: Opaque
stringData:
  POLYMARKET_API_KEY: "your-api-key"
  POLYMARKET_SECRET: "your-secret"
  POLYMARKET_PRIVATE_KEY: "your-private-key"
  POLYMARKET_WS_URL: "wss://ws-subscriptions-clob.polymarket.com/ws/market"
  MIN_PROFIT_THRESHOLD: "0.02"
  MAX_POSITION_SIZE: "100.0"
  TRADE_AMOUNT: "10.0"
  RUST_LOG: "info"

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: polymarket-bot
spec:
  replicas: 1
  selector:
    matchLabels:
      app: polymarket-bot
  template:
    metadata:
      labels:
        app: polymarket-bot
    spec:
      containers:
      - name: bot
        image: polymarket-bot:latest
        envFrom:
        - secretRef:
            name: polymarket-bot-config
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          exec:
            command:
            - pgrep
            - polymarket
          initialDelaySeconds: 30
          periodSeconds: 30
        restartPolicy: Always
```

Deploy:

```bash
kubectl apply -f k8s-deployment.yaml
kubectl logs -f deployment/polymarket-bot
```

## Cloud Deployment

### AWS EC2

1. Launch EC2 instance (t3.micro or larger)
2. Install Rust and dependencies
3. Clone repository
4. Configure with secrets manager or environment
5. Set up systemd service
6. Configure CloudWatch for logs

```bash
# Install dependencies
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone <repo-url>
cd polymarket-arbitrage-bot
cargo build --release

# Set up service (see systemd section)
```

### Google Cloud Platform (GCP)

1. Create Compute Engine instance
2. Use startup script to install and configure
3. Store secrets in Secret Manager
4. Set up Cloud Logging

```bash
#!/bin/bash
# Startup script

apt-get update
apt-get install -y build-essential pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Clone and build
cd /opt
git clone <repo-url>
cd polymarket-arbitrage-bot
cargo build --release

# Fetch secrets from Secret Manager
# ... configure environment ...

# Start service
./target/release/polymarket-arbitrage-bot
```

### DigitalOcean Droplet

1. Create droplet (1GB RAM minimum)
2. Follow local deployment steps
3. Set up systemd service
4. Configure monitoring

### Heroku

Create `Procfile`:

```
worker: ./target/release/polymarket-arbitrage-bot
```

Create `rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.70"
```

Deploy:

```bash
heroku create your-bot-name
heroku buildpacks:set emk/rust
heroku config:set RUST_LOG=info
heroku config:set POLYMARKET_API_KEY=your-key
# ... set other config vars ...
git push heroku main
heroku ps:scale worker=1
heroku logs --tail
```

## Monitoring and Logging

### Structured Logging

The bot uses `env_logger`. Configure log level:

```bash
# Show all logs
RUST_LOG=debug ./polymarket-arbitrage-bot

# Show only warnings and errors
RUST_LOG=warn ./polymarket-arbitrage-bot

# Module-specific logging
RUST_LOG=polymarket_arbitrage_bot=debug,reqwest=warn ./polymarket-arbitrage-bot
```

### Log Rotation

For systemd:

```bash
sudo nano /etc/systemd/system/polymarket-bot.service
```

Add:

```ini
[Service]
StandardOutput=append:/var/log/polymarket-bot/output.log
StandardError=append:/var/log/polymarket-bot/error.log
```

Configure logrotate:

```bash
sudo nano /etc/logrotate.d/polymarket-bot
```

```
/var/log/polymarket-bot/*.log {
    daily
    missingok
    rotate 7
    compress
    delaycompress
    notifempty
    create 0640 your-user your-user
    sharedscripts
    postrotate
        systemctl reload polymarket-bot > /dev/null 2>&1 || true
    endscript
}
```

### Monitoring with Prometheus

Extend the bot to expose metrics (future enhancement):

```rust
// Add prometheus crate
use prometheus::{Encoder, TextEncoder, Counter, Gauge};

// Define metrics
lazy_static! {
    static ref TRADES_EXECUTED: Counter = 
        register_counter!("trades_executed_total", "Total trades").unwrap();
    static ref CURRENT_PNL: Gauge = 
        register_gauge!("current_pnl", "Current PnL").unwrap();
}

// Expose metrics endpoint
// ... serve on HTTP port ...
```

## Performance Tuning

### System Limits

Increase file descriptors for WebSocket connections:

```bash
# Temporary
ulimit -n 4096

# Permanent
sudo nano /etc/security/limits.conf
```

Add:

```
your-user soft nofile 4096
your-user hard nofile 8192
```

### Network Optimization

For low-latency execution:

```bash
# Disable TCP slow start
sudo sysctl -w net.ipv4.tcp_slow_start_after_idle=0

# Increase TCP buffer sizes
sudo sysctl -w net.core.rmem_max=16777216
sudo sysctl -w net.core.wmem_max=16777216
```

### Rust Optimization

Build with additional optimizations:

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

## Backup and Recovery

### State Backup

If implementing persistent state (future):

```bash
# Backup
tar -czf polymarket-bot-backup-$(date +%Y%m%d).tar.gz \
    /path/to/bot/data

# Restore
tar -xzf polymarket-bot-backup-YYYYMMDD.tar.gz -C /path/to/bot/
```

### Configuration Backup

```bash
# Backup environment
cp .env .env.backup.$(date +%Y%m%d)

# Store encrypted
gpg -c .env
```

## Security Hardening

### 1. API Key Protection

```bash
# Use restrictive permissions
chmod 600 .env

# Store in secrets manager (AWS)
aws secretsmanager create-secret \
    --name polymarket-bot-config \
    --secret-string file://.env
```

### 2. Network Isolation

```bash
# Allow only outbound HTTPS
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow 22/tcp  # SSH only
sudo ufw enable
```

### 3. User Isolation

```bash
# Run as dedicated user
sudo useradd -r -s /bin/false polymarket-bot
sudo chown -R polymarket-bot:polymarket-bot /opt/polymarket-arbitrage-bot

# Update systemd service
User=polymarket-bot
Group=polymarket-bot
```

## Health Checks

### Script-based Health Check

Create `health_check.sh`:

```bash
#!/bin/bash

# Check if process is running
if ! pgrep -f polymarket-arbitrage-bot > /dev/null; then
    echo "Bot is not running!"
    # Restart or alert
    systemctl restart polymarket-bot
    exit 1
fi

# Check log for recent activity (within last 5 minutes)
if ! find /var/log/polymarket-bot/output.log -mmin -5 | grep -q .; then
    echo "No recent activity!"
    exit 1
fi

echo "Bot is healthy"
exit 0
```

Run periodically:

```bash
# Add to crontab
*/5 * * * * /path/to/health_check.sh
```

## Troubleshooting

### Bot won't start

```bash
# Check logs
journalctl -u polymarket-bot -n 100

# Verify configuration
cat .env

# Test manually
RUST_LOG=debug ./target/release/polymarket-arbitrage-bot
```

### WebSocket disconnections

- Check network stability
- Verify firewall allows WSS
- Check Polymarket service status
- Bot auto-reconnects with backoff

### No trades executing

- Verify markets exist
- Check profit threshold settings
- Ensure API credentials are valid
- Review logs for errors

## Maintenance

### Updating the Bot

```bash
# Stop service
sudo systemctl stop polymarket-bot

# Pull updates
git pull origin main

# Rebuild
cargo build --release

# Restart
sudo systemctl start polymarket-bot
```

### Database Cleanup (if implemented)

```bash
# Archive old trade data
# Compress logs
# Clear temporary files
```

## Cost Optimization

### AWS Cost Reduction
- Use spot instances for non-critical testing
- t3.micro sufficient for single bot
- CloudWatch logs can be expensive - use filtering

### Resource Usage
- Bot is lightweight: ~50MB RAM typical
- Minimal CPU usage
- Network bandwidth: <1GB/day typically

## Scaling

### Horizontal Scaling
- Run multiple instances for different market pairs
- Use different API keys per instance
- Coordinate via shared state (Redis/Database)

### Vertical Scaling
- Increase instance size for more markets
- Monitor WebSocket connection limits
- Consider API rate limits

---

For additional help, refer to README.md and ARCHITECTURE.md
