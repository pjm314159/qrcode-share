# QRcode Share

Real-time link sharing through QR codes. Create a channel, scan or paste a link, and everyone in the channel receives it instantly via WebSocket.

## Features

- Create password-protected or public channels
- Share links via QR code scan or paste
- Real-time message delivery through WebSocket
- Auto-open links on click or on receive
- Domain-based link limitation per channel
- WeChat in-app browser QR scanning support
- Responsive design (desktop + mobile)

## Tech Stack

| Layer | Technology |
|---|---|
| Frontend | React 19, TypeScript 6, Vite 8, Tailwind CSS 4 |
| State | Zustand 5 |
| Backend | Rust, Axum 0.7, Tokio |
| Database | PostgreSQL 16 |
| Deployment | Docker Compose |

## Quick Start

### Docker (Recommended)

```bash
cp .env.example .env
docker compose up -d
```

The app will be available at `http://localhost`.

### Local Development

**Backend:**

```bash
cd qrcode_share_backend
cp .env.example .env
cargo run
```

**Frontend:**

```bash
cd qrcode_share_fronted
pnpm install
pnpm dev
```

## Project Structure

```
qrcode_share/
  qrcode_share_backend/     # Rust/Axum API server
    src/
      handlers/             # HTTP + WebSocket handlers
      db/                   # Database repository layer
      models/               # Data models
      middleware/            # Rate limiting, security
      state/                # App state, channel state, metrics
      tasks/                # Background cleanup tasks
    tests/                  # Integration + WebSocket tests
    migrations/             # SQL migration files
  qrcode_share_fronted/     # React SPA
    src/
      api/                  # HTTP client + WebSocket
      components/           # UI components (channel, message, icons, etc.)
      hooks/                # Custom React hooks
      pages/                # Route pages
      stores/               # Zustand state stores
      types/                # TypeScript definitions
      utils/                # Helper functions
  docs/                     # Documentation
  scripts/                  # Deploy + test scripts
  deploy/                   # Systemd service file
```

## API

### REST

| Method | Path | Description |
|---|---|---|
| POST | `/api/channels` | Create channel |
| GET | `/api/channels/:id` | Get channel |
| GET | `/api/channels` | List channels |
| DELETE | `/api/channels/:id` | Delete channel |
| POST | `/api/channels/:id/messages` | Send message |
| GET | `/api/channels/:id/messages` | Get messages |
| GET | `/api/system/stats` | System stats |

### WebSocket

Connect to `ws://host/ws/channel/{id}?password={optional}` to receive messages in real-time.

## Configuration

All configuration is via environment variables. Copy `.env.example` to `.env` and customize:

| Variable | Default         | Description |
|---|-----------------|---|
| `DB_PASSWORD` | `your-password` | PostgreSQL password |
| `PORT` | `3000`          | Backend port |
| `FRONTEND_PORT` | `80`            | Nginx port |
| `MAX_CHANNELS` | `5000`          | Maximum channels |
| `MAX_MESSAGES_PER_CHANNEL` | `300`           | Messages per channel |
| `MESSAGE_TTL_SECONDS` | `3600`          | Message lifetime |
| `MAX_CONNECTIONS` | `500`           | Max WebSocket connections |
| `CORS_ORIGINS` | `*`             | Allowed origins |
| `WX_APPID` | -               | WeChat App ID (optional) |
| `WX_SECRET` | -               | WeChat App Secret (optional) |

## Testing

**Frontend:**

```bash
cd qrcode_share_fronted
pnpm test              # Vitest watch mode
pnpm test -- --run     # Single run
pnpm test:coverage     # With coverage
pnpm lint              # ESLint
pnpm build             # Type-check + build
```

**Backend:**

```bash
cd qrcode_share_backend
cargo test                           # Unit + integration tests
cargo test --test integration_tests  # Integration only
cargo clippy                         # Lint
```

## Documentation

| Document | Path |
|---|---|
| Frontend Summary | `docs/en/fronted.md` |
| Design System | `docs/DESIGN.md` |
| Image Prompts | `docs/en/images.md` |
| API Reference | `docs/en/api.md` |
| Database Schema | `docs/en/database.md` |
| Performance Guide | `docs/en/performance_improve.md` |
| Deployment Guide | `docs/en/configure_and_deploy.md` |
| Task Plan | `docs/en/task.md` |
| Backend Tests | `docs/en/test.md` |

## License

MIT
