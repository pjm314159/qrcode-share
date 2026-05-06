# Frontend Development Documentation Summary

This document consolidates all frontend-related development documentation into a single reference.

---

## Document Index

| Document | Path | Description |
|---|---|---|
| Overview | `docs/en/overview.md` | Project overview and architecture |
| Design System | `docs/DESIGN.md` | Clay design system tokens and guidelines |
| Phase 1 Refactoring | `docs/en/refactore-fronted.md` | Initial refactoring: SVG icons, design tokens, dark mode removal |
| Phase 2 Refactoring | `docs/en/refactoring-fronted-2.md` | Visual enhancement, component decomposition, progressive disclosure |
| Image Prompts | `docs/en/images.md` | AI image generation prompts for all illustrations |
| Frontend Test | `docs/en/fronted_test.md` | Frontend testing strategy (empty placeholder) |
| Backend Test | `docs/en/test.md` | Backend test documentation (153 tests) |
| API | `docs/en/api.md` | REST API and WebSocket protocol specification |
| Database | `docs/en/database.md` | Database schema and migration guide |
| Performance | `docs/en/performance_improve.md` | Performance optimization for 2-core/2GB server |
| Deployment | `docs/en/configure_and_deploy.md` | Docker deployment and configuration guide |
| Task Plan | `docs/en/task.md` | Development task plan with phase tracking |

---

## 1. Project Overview

**QRcode Share** is a real-time link sharing application built around QR codes. Users create channels, share links via QR scan or paste, and receive links in real-time through WebSocket connections.

### Tech Stack

| Layer | Technology |
|---|---|
| Frontend | React 19 + TypeScript 6 + Vite 8 |
| Styling | Tailwind CSS 4 (Clay design system) |
| State | Zustand 5 |
| Routing | React Router 7 |
| HTTP | Axios |
| QR | html5-qrcode + qrcode |
| Testing | Vitest 3 + @testing-library/react |
| Linting | ESLint 10 + Prettier |
| Backend | Rust (Axum 0.7 + Tokio) |
| Database | PostgreSQL 16 |
| Deployment | Docker Compose |

### Frontend Scripts

```bash
pnpm dev          # Start dev server
pnpm build        # Type-check + production build
pnpm lint         # ESLint check
pnpm test         # Run tests
pnpm test:ui      # Vitest UI
pnpm test:coverage # Coverage report
pnpm format       # Prettier format
pnpm format:check # Prettier check
```

---

## 2. Design System (Clay)

The frontend uses a custom **Clay** design system with warm, hand-crafted aesthetics inspired by claymation.

### Color Tokens

| Token | Value | Usage |
|---|---|---|
| `canvas` | `#fffaf0` | Page background |
| `ink` | `#0a0a0a` | Primary text |
| `ink-active` | `#1a1a1a` | Active/hover text |
| `body` | `#3d3d3d` | Body text |
| `muted` | `#6b6b6b` | Secondary text |
| `muted-soft` | `#9a9a9a` | Tertiary text |
| `hairline` | `#e8e0d0` | Borders |
| `surface-soft` | `#faf5e8` | Subtle backgrounds |
| `surface-card` | `#f5f0e0` | Card backgrounds |
| `surface-strong` | `#e8e0d0` | Skeleton/strong surface |
| `on-primary` | `#fffaf0` | Text on ink/brand backgrounds |
| `brand-pink` | `#ff4d8b` | Primary accent |
| `brand-teal` | `#1a3a3a` | Secondary accent |
| `brand-lavender` | `#b8a4ed` | Tertiary accent |
| `brand-peach` | `#ffb084` | Warm accent |
| `brand-ochre` | `#e8b94a` | Gold accent |
| `brand-mint` | `#a4d4c5` | Fresh accent |
| `brand-coral` | `#ff6b5a` | Alert accent |
| `success` | `#22c55e` | Success state |
| `warning` | `#eab308` | Warning state |
| `error` | `#ef4444` | Error state |

### Typography

- Font: System font stack (`font-sans`)
- Headings: `font-bold`
- Body: `text-body` color
- Muted: `text-muted` / `text-muted-soft`

### Component Variants

**Card variants:** `cream`, `feature-pink`, `feature-teal`, `feature-lavender`, `feature-peach`

**Button variants:** `primary`, `secondary`, `danger`

### Animation Tokens

| Token | Duration | Usage |
|---|---|---|
| `animate-fade-in` | 0.3s | Page transitions |
| `animate-slide-up` | 0.3s | Bottom sheet entry |
| `animate-slide-down` | 0.2s | Progressive disclosure expand |
| `animate-slide-up-sm` | 0.2s | Content stagger |
| `animate-pulse-slow` | 3s | Gentle breathing effect |

### Accessibility

- `prefers-reduced-motion` media query disables all animations
- All decorative elements have `aria-hidden="true"`
- Interactive elements have proper ARIA attributes
- Touch targets are at least 44x44px

---

## 3. Refactoring History

### Phase 1: Foundation (refactore-fronted.md)

**Status:** Completed

Key changes:
- Replaced all emoji characters with SVG icons (28 custom icons)
- Applied Clay design system tokens throughout
- Removed dark mode (single warm light theme)
- Created reusable UI components (Button, Input, Card, Select)
- Established project structure (components/, pages/, stores/, hooks/)

### Phase 2: Visual Enhancement (refactoring-fronted-2.md)

**Status:** Completed

Key changes:
- **DecorativeBlob component**: Soft blurred background shapes for visual warmth
- **Skeleton component**: Replaces spinners for better perceived performance
- **Toggle component**: Accessible switch with `role="switch"` + `aria-checked`
- **ChannelPage decomposition**: 836-line monolith split into 4 focused components
  - `ChatMessageCard` -- single message with countdown
  - `ChatMessageList` -- message list with pagination
  - `MessageEditor` -- bottom sheet for sending links
  - `ChannelSettingsPanel` -- slide-out settings
- **HomePage hero enhancement**: Gradient background, decorative blobs, title accent line, centered illustration
- **CreateChannelForm progressive disclosure**: Required fields only by default, expandable "Advanced options"
- **ChannelPage bottom bar**: Two-button layout with brand-pink and brand-teal styling
- **Micro-interactions**: `active:scale-[0.98]` on cards/buttons, smooth transitions
- **CreatePage/NotFoundPage**: Decorative blob backgrounds
- New animation tokens: `animate-slide-down`, `animate-slide-up-sm`
- `prefers-reduced-motion` support

---

## 4. Component Architecture

### Directory Structure

```
src/
  api/              # HTTP client, channel/message/system APIs, WebSocket
  components/
    channel/        # ChannelCard, ChannelList, ChannelSearch,
                    # CreateChannelForm, JoinChannelForm,
                    # PasswordModal, ChannelSettingsPanel
    common/         # Loading, ErrorBoundary, ErrorMessage,
                    # EmptyState, ScrollToTop, PageTransition,
                    # DecorativeBlob, Skeleton
    icons/          # 28 SVG icon components (Icon.tsx + icons/)
    layout/         # Layout (header + footer + outlet)
    message/        # ChatMessageCard, ChatMessageList,
                    # MessageEditor, MessageCard, MessageList,
                    # SendMessageForm, CountdownTimer, LinkWarning
    qrcode/         # QRCodeDisplay, QRScanner, SmartQRScanner
    ui/             # Button, Input, Card, Select, Toggle
    wechat/         # WechatProvider, WechatScanner
  constants/        # Image paths
  hooks/            # useLinkWarning, useTimer, useWebSocket, useWechat
  pages/            # HomePage, ChannelPage, ChannelListPage,
                    # CreatePage, NotFoundPage
  stores/           # channelStore, messageStore, connectionStore,
                    # settingsStore (all Zustand)
  types/            # TypeScript type definitions
  utils/            # helpers, wechat utilities
```

### Page-Component Mapping

| Page | Key Components |
|---|---|
| HomePage | DecorativeBlob, CreateChannelForm, JoinChannelForm, Card (feature variants) |
| ChannelPage | ChatMessageList, MessageEditor, ChannelSettingsPanel, LinkWarning |
| ChannelListPage | ChannelSearch, ChannelList, ChannelCard |
| CreatePage | DecorativeBlob, CreateChannelForm |
| NotFoundPage | DecorativeBlob, Button |

### State Management (Zustand)

| Store | Responsibility |
|---|---|
| `channelStore` | Current channel, channel CRUD, pagination |
| `messageStore` | Messages, send/receive, WebSocket integration |
| `connectionStore` | WebSocket connection state, subscriber count |
| `settingsStore` | Auto-open links, auto-open received links |

---

## 5. Image Assets

All images are located in `public/images/` and follow the Clay 3D claymation aesthetic.

### Active Images

| Image | Size | Location |
|---|---|---|
| `hero-illustration.png` | 800x600 | HomePage hero section |
| `logo-mark.png` | 256x256 | Layout header, favicon |
| `feature-scan.png` | 320x240 | Scan feature card |
| `feature-share.png` | 320x240 | Share feature card |
| `feature-open.png` | 320x240 | Open feature card |
| `feature-create.png` | 320x240 | Create feature card |
| `not-found.png` | 320x240 | 404 page |
| `empty-inbox.png` | 320x240 | Empty message state |
| `footer-mountains.png` | 1280x200 | Footer decoration |
| `password-lock.png` | 240x180 | Password modal |

### Deprecated Images (Phase 2)

| Image | Replacement |
|---|---|
| `status-connected.png` | `<IconSuccess>` SVG icon |
| `status-connecting.png` | `<IconConnecting>` SVG icon |
| `status-disconnected.png` | `<IconDisconnected>` SVG icon |
| `2-64x64.png` | `logo-mark.png` at appropriate size |

### Image Style Guidelines

- 3D claymation aesthetic with soft studio lighting
- Warm cream canvas background (#fffaf0) or transparent PNG
- Brand colors only from Clay palette
- No text, no people, no emojis
- Hand-crafted, friendly, approachable feel

Detailed prompts for regenerating images are in `docs/en/images.md`.

---

## 6. Testing

### Test Framework

- **Vitest 3** with jsdom environment
- **@testing-library/react** for component testing
- **@testing-library/jest-dom/vitest** for DOM assertions
- Setup file: `src/test/setup.ts`

### Test Statistics (Phase 2)

| Category | Tests | Files |
|---|---|---|
| Components | 328 | 27 |
| API | ~20 | 4 |
| Stores | ~15 | 2 |
| Utils | ~10 | 2 |
| Pages | ~30 | 2 |

### Test Conventions

- Test files colocated in `__tests__/` directories
- Mock external dependencies (stores, API, hooks)
- Use `fireEvent` for interactions
- Use `vi.fn()` for mock functions
- TDD cycle: Red -> Green -> Refactor

### Running Tests

```bash
pnpm test              # Watch mode
pnpm test -- --run     # Single run
pnpm test:coverage     # With coverage
pnpm test:ui           # Vitest UI
```

---

## 7. API Integration

### REST Endpoints

| Method | Path | Description |
|---|---|---|
| POST | `/api/channels` | Create channel |
| GET | `/api/channels/:id` | Get channel |
| GET | `/api/channels` | List channels (paginated) |
| DELETE | `/api/channels/:id` | Delete channel |
| POST | `/api/channels/:id/messages` | Send message |
| GET | `/api/channels/:id/messages` | Get messages (paginated) |
| GET | `/api/system/stats` | System statistics |

### WebSocket Protocol

- Endpoint: `ws://host/ws/channel/{id}?password={optional}`
- Client sends: JSON messages with `name`, `link`, `expire_seconds`
- Server broadcasts: JSON messages with `id`, `name`, `link`, `expire_at`, `message_type`, `created_at`
- Heartbeat: Server sends `ping`, client responds with `pong`

---

## 8. Build and Deployment

### Frontend Build

```bash
cd qrcode_share_fronted
pnpm install
pnpm build    # Output: dist/
```

The build output is served by Nginx in the Docker container.

### Docker Deployment

```bash
# From project root
cp .env.example .env
docker compose up -d
```

Services:
- `postgres` -- PostgreSQL 16 database
- `migrate` -- Runs SQL migrations on startup
- `backend` -- Rust/Axum API server (port 3000)
- `frontend` -- Nginx serving static build + reverse proxy to backend (port 80)

### Environment Variables

Key frontend-related variables:
- `FRONTEND_PORT` -- Nginx port (default: 80)
- `CORS_ORIGINS` -- Allowed origins (default: *)
- `WX_APPID` / `WX_SECRET` -- WeChat JS-SDK (optional)

---

## 9. Performance Targets

| Metric | Target |
|---|---|
| Message delivery latency | < 50ms (p95) |
| WebSocket connection | < 100ms |
| Concurrent connections | 300-500 |
| Memory usage | 400-800MB |
| CPU usage | 40-70% average |

Frontend-specific:
- Skeleton screens for perceived performance
- Lazy-loaded QR scanner (dynamic import)
- GPU-accelerated blur for decorative blobs
- `prefers-reduced-motion` for accessibility
