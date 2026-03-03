# ── Dev stage (target for docker-compose.override.yml) ──────────────
FROM node:22-alpine AS dev

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

# Sources are volume-mounted in dev, not copied
CMD ["npm", "run", "dev"]

# ── Builder stage (production build) ────────────────────────────────
FROM node:22-alpine AS builder

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build

# ── Runtime stage (minimal production image) ────────────────────────
FROM node:22-alpine AS runtime

WORKDIR /app

RUN addgroup --system app && adduser --system --ingroup app app

COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static
COPY --from=builder /app/public ./public

USER app

ENV PORT=3001
EXPOSE 3001

CMD ["node", "server.js"]
