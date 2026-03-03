# Offrii

Wishlist and gift platform — create wishlists, share them, and let others contribute to your wishes.

## Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust (Axum, Tokio) |
| Frontend | Next.js 16, TypeScript, Tailwind CSS 4, shadcn/ui |
| Database | PostgreSQL |
| Cache | Redis |

## Monorepo Structure

```
offrii/
├── backend/              # Rust workspace
│   ├── crates/
│   │   ├── api/          # HTTP layer (Axum routes, middleware)
│   │   ├── domain/       # Business logic (zero infra deps)
│   │   └── infra/        # Database, cache, external services
│   └── migrations/       # SQL migrations
└── frontend/             # Next.js 16 app
    ├── app/              # App Router pages
    ├── components/ui/    # shadcn/ui components
    └── lib/              # Utilities
```

## Getting Started

### Prerequisites

- Rust (latest stable)
- Node.js 22+
- PostgreSQL 16
- Redis 7

### Backend

```bash
cd backend
cargo build
cargo run -p api   # starts on http://localhost:3000
```

Health check: `GET /health` → `200 ok`

### Frontend

```bash
cd frontend
npm install
npm run dev        # starts on http://localhost:3001
```

### Environment Variables

Copy `.env.example` to `.env` and adjust values:

```bash
cp .env.example .env
```

## License

MIT
