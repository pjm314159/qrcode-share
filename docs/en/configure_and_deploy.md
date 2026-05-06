# Configuration & Deployment Guide

## Prerequisites

- Docker 24+ and Docker Compose v2+
- 1GB+ RAM, 2+ CPU cores
- Ports 80 (or custom) available

## Quick Start

### 1. Clone the Repository

```bash
git clone <repo-url> qrcode_share
cd qrcode_share
```

### 2. Configure Environment

```bash
cp .env.example .env
```

Edit `.env` to customize settings. All variables have sensible defaults — you only need to override what you want to change:

```env
# Database
DB_PASSWORD=yourPassword

# Backend Server
RUST_LOG=info,qrcode_share_backend=debug

# Channel & Message Limits
MAX_CHANNELS=5000
MAX_MESSAGES_PER_CHANNEL=300
MESSAGE_TTL_SECONDS=3600
CHANNEL_TTL_DAYS=30

# Connection Limits
MAX_CONNECTIONS=500
MAX_CONNECTIONS_PER_CHANNEL=50

# Rate Limiting
MAX_MESSAGES_PER_MINUTE=60
MAX_CHANNELS_PER_USER=10

# Performance Tuning
CLEANUP_INTERVAL_SECONDS=120
HEARTBEAT_INTERVAL_SECONDS=30
BROADCAST_BUFFER_SIZE=256

# CORS (comma-separated origins, or * for any)
CORS_ORIGINS=*

# Frontend
FRONTEND_PORT=80

# WeChat JS-SDK (optional — see WeChat section below)
# WX_APPID=your_wechat_app_id
# WX_SECRET=your_wechat_app_secret
```

### 3. Start All Services

```bash
docker compose up -d
```

This will start:
- **postgres** — PostgreSQL 16 database
- **migrate** — Runs database migrations, then exits
- **backend** — Rust/Axum API server (port 3000, internal)
- **frontend** — Nginx serving React app + reverse proxy (port 80, external)

### 4. Verify

```bash
# Check all services are running
docker compose ps

# Test health endpoint
curl http://localhost/health

# Expected response:
# {"status":"healthy","version":"0.1.0","uptime_seconds":...}
```

Open `http://localhost` in your browser to use the application.

---

## Architecture

```
                    ┌─────────────┐
                    │   Browser   │
                    └──────┬──────┘
                           │ :80
                    ┌──────▼──────┐
                    │   Nginx     │  ← frontend container
                    │  (React +   │
                    │   Proxy)    │
                    └──────┬──────┘
                           │ :3000
              ┌────────────▼────────────┐
              │   Backend (Rust/Axum)   │  ← backend container
              │   - REST API            │
              │   - WebSocket           │
              │   - In-memory state     │
              └────────────┬────────────┘
                           │ :5432
              ┌────────────▼────────────┐
              │   PostgreSQL 16         │  ← postgres container
              │   - Channel metadata    │
              └─────────────────────────┘
```

---

## Configuration Reference

All configuration is managed through a single `.env` file at the project root. Copy `.env.example` to get started.

### Core Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DB_PASSWORD` | `Qwer314159` | PostgreSQL password (change in production!) |
| `FRONTEND_PORT` | `80` | External port for the frontend |
| `CORS_ORIGINS` | `*` | Allowed CORS origins (comma-separated) |
| `RUST_LOG` | `info,qrcode_share_backend=debug` | Log level |

### Backend Server

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Server bind address |
| `PORT` | `3000` | Server bind port |
| `DATABASE_URL` | (auto) | PostgreSQL connection string (auto-generated in Docker) |
| `DB_MAX_CONNECTIONS` | `5` | Database connection pool max |
| `DB_MIN_CONNECTIONS` | `1` | Database connection pool min |

### Channel & Message Limits

| Variable | Default | Description |
|----------|---------|-------------|
| `MAX_CHANNELS` | `5000` | Maximum total channels |
| `MAX_MESSAGES_PER_CHANNEL` | `300` | Max messages per channel |
| `MAX_MESSAGE_SIZE` | `5120` | Max single message size (bytes) |
| `MESSAGE_TTL_SECONDS` | `3600` | Message lifetime (1 hour) |
| `CHANNEL_TTL_DAYS` | `30` | Channel lifetime without activity |

### Connection Limits

| Variable | Default | Description |
|----------|---------|-------------|
| `MAX_CONNECTIONS` | `500` | Maximum WebSocket connections |
| `MAX_CONNECTIONS_PER_CHANNEL` | `50` | Max connections per channel |

### Rate Limiting

| Variable | Default | Description |
|----------|---------|-------------|
| `MAX_MESSAGES_PER_MINUTE` | `60` | Messages per minute per user |
| `MAX_CHANNELS_PER_USER` | `10` | Channels per user per hour |

### Performance Tuning

| Variable | Default | Description |
|----------|---------|-------------|
| `CLEANUP_INTERVAL_SECONDS` | `120` | How often to run cleanup |
| `HEARTBEAT_INTERVAL_SECONDS` | `30` | WebSocket heartbeat interval |
| `CONNECTION_TIMEOUT_SECONDS` | `60` | WebSocket connection timeout |
| `BROADCAST_BUFFER_SIZE` | `256` | Broadcast channel buffer size |

### Frontend

| Variable | Default | Description |
|----------|---------|-------------|
| `FRONTEND_PORT` | `80` | External port for the frontend |
| `VITE_API_URL` | (auto) | Backend API URL (auto-detected in Docker) |

---

## WeChat JS-SDK Setup (Optional)

The app supports QR code scanning via WeChat JS-SDK when opened in the WeChat browser. If not configured, the app automatically falls back to the standard browser QR scanner.

### Prerequisites

1. A verified **WeChat Official Account** (服务号) — subscription accounts (订阅号) do not have JS-SDK scan permissions
2. Your server's **public IP** added to the WeChat IP whitelist
3. Your **domain** configured as a JS-SDK safe domain

### Configuration Steps

1. Log in to [WeChat Official Account Platform](https://mp.weixin.qq.com)
2. Go to **Settings > Basic > IP Whitelist** and add your server IP
3. Go to **Settings > JS-SDK Safe Domain** and add your domain
4. Copy the **AppID** and **AppSecret** from **Settings > Basic**
5. Add to your `.env`:

```env
WX_APPID=wx1234567890abcdef
WX_SECRET=your_wechat_app_secret_here
```

6. Restart the backend:

```bash
docker compose up -d --build backend
```

### Verification

The backend automatically verifies WeChat configuration on startup. Check the logs:

```bash
docker compose logs backend | grep -i wechat
```

**Success:**
```
INFO  WeChat JS-SDK: Verifying configuration with APPID=wx1234567890abcdef
INFO  WeChat JS-SDK: access_token obtained successfully
INFO  WeChat JS-SDK: Configuration verified, jsapi_ticket obtained, WeChat features enabled
```

**Common errors:**

| Log message | Cause | Fix |
|-------------|-------|-----|
| `WX_APPID not configured` | Missing `WX_APPID` in `.env` | Add `WX_APPID` to `.env` |
| `errcode=40013, errmsg=invalid appid` | Wrong AppID | Check `WX_APPID` value |
| `errcode=40125, errmsg=invalid appsecret` | Wrong AppSecret | Check `WX_SECRET` value |
| `errcode=40164, errmsg=invalid ip` | Server IP not whitelisted | Add server IP to WeChat IP whitelist |
| `errcode=40048, errmsg=invalid url domain` | Domain not in safe domains | Add domain to JS-SDK safe domains |

You can also check the status via API:

```bash
curl http://localhost/api/wechat/status
# {"available":true,"reason":null}  — working
# {"available":false,"reason":"WX_APPID not configured"}  — not configured
```

---

## Docker Compose Services

### Service: `postgres`

- **Image**: `postgres:16-alpine`
- **Volume**: `postgres_data` (persistent)
- **Health check**: `pg_isready`
- **Resource limits**: 1 CPU, 512MB RAM

### Service: `migrate`

- **Image**: `postgres:16-alpine`
- **Purpose**: Runs SQL migrations from `/migrations/`, then exits
- **Depends on**: `postgres` (healthy)

### Service: `backend`

- **Build**: `./qrcode_share_backend/Dockerfile` (multi-stage Rust build)
- **Health check**: `wget http://localhost:3000/health`
- **Resource limits**: 2 CPUs, 1800MB RAM
- **Depends on**: `migrate` (completed successfully)

### Service: `frontend`

- **Build**: `./qrcode_share_fronted/Dockerfile` (Node build + Nginx serve)
- **Ports**: `${FRONTEND_PORT:-80}:80`
- **Health check**: `wget http://localhost:80/`
- **Resource limits**: 0.5 CPU, 256MB RAM
- **Depends on**: `backend` (healthy)

---

## Common Operations

### View Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f backend
docker compose logs -f frontend
```

### Restart a Service

```bash
docker compose restart backend
```

### Rebuild After Code Changes

```bash
# Rebuild and restart all
docker compose up -d --build

# Rebuild only backend
docker compose up -d --build backend
```

### Database Operations

```bash
# Connect to PostgreSQL
docker compose exec postgres psql -U qrcode -d qrcode_share

# Backup database
docker compose exec postgres pg_dump -U qrcode qrcode_share > backup.sql

# Restore database
cat backup.sql | docker compose exec -T postgres psql -U qrcode qrcode_share
```

### Scale Considerations

For higher traffic, adjust resource limits in `docker-compose.yml`:

```yaml
backend:
  deploy:
    resources:
      limits:
        cpus: "4"
        memory: 4G
```

---

## SSL/HTTPS Setup

### Option 1: Let's Encrypt with Certbot

1. Install certbot on the host:
   ```bash
   apt install certbot
   ```

2. Obtain a certificate:
   ```bash
   certbot certonly --standalone -d your-domain.com
   ```

3. Update `nginx.conf` to add SSL:
   ```nginx
   server {
       listen 443 ssl;
       ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
       ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;
       # ... rest of config
   }
   ```

4. Mount certificates in `docker-compose.yml`:
   ```yaml
   frontend:
     volumes:
       - /etc/letsencrypt:/etc/letsencrypt:ro
   ```

5. Set up auto-renewal:
   ```bash
   echo "0 0 * * * certbot renew --quiet && docker compose restart frontend" | crontab -
   ```

### Option 2: Reverse Proxy with External Nginx/Caddy

If you already have a reverse proxy on the host:

1. Set `FRONTEND_PORT=8080` (or any internal port)
2. Configure your external proxy to forward to that port

---

## Monitoring

### Health Check

```bash
curl http://localhost/health
```

Response:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 86400,
  "checks": {
    "memory": { "used_mb": 500, "limit_mb": 2000, "percentage": 25.0, "healthy": true },
    "channels": { "count": 10, "limit": 5000, "percentage": 0.2, "healthy": true }
  }
}
```

### Metrics

```bash
curl http://localhost/metrics
```

Response:
```json
{
  "channels": { "total": 42, "active": 38 },
  "messages": { "total": 15234, "per_channel_avg": 362.7 },
  "connections": { "active_websocket": 87, "total_subscribers": 150 },
  "system": { "memory_used_mb": 500, "memory_limit_mb": 2000, "uptime_seconds": 86400 }
}
```

---

## Troubleshooting

### Backend won't start

```bash
# Check backend logs
docker compose logs backend

# Common issues:
# - Database not ready: wait for postgres health check
# - Migration failed: check migrate logs with `docker compose logs migrate`
```

### WebSocket connections fail

- Ensure nginx WebSocket proxy is configured (already in `nginx.conf`)
- Check `MAX_CONNECTIONS` and `MAX_CONNECTIONS_PER_CHANNEL` limits
- Verify the client connects to `/api/channels/:id/ws`

### Frontend shows blank page

- Check browser console for errors
- Ensure backend is healthy: `curl http://localhost/health`
- Check nginx logs: `docker compose logs frontend`

### Database connection errors

```bash
# Verify PostgreSQL is running
docker compose exec postgres pg_isready

# Check connection string
docker compose exec backend printenv DATABASE_URL
```

### WeChat QR scanning not working

- Check `/api/wechat/status` endpoint
- Review backend startup logs for WeChat errors
- Ensure your server IP is whitelisted in WeChat settings
- Ensure your domain is configured as JS-SDK safe domain
- If WeChat is not configured, the app automatically uses the standard browser QR scanner

---

## Production Checklist

- [ ] Change `DB_PASSWORD` from default
- [ ] Set `CORS_ORIGINS` to your actual domain(s)
- [ ] Configure SSL/HTTPS
- [ ] Set `RUST_LOG=info` (not debug) for production
- [ ] Adjust resource limits based on expected traffic
- [ ] Set up database backups
- [ ] Configure monitoring/alerting on `/health` endpoint
- [ ] Review and adjust rate limiting parameters
- [ ] Configure WeChat JS-SDK if targeting WeChat users
