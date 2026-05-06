FROM node:22-alpine AS builder

RUN corepack enable && corepack prepare pnpm@latest --activate

WORKDIR /app

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./

RUN pnpm install --frozen-lockfile

COPY . .

RUN pnpm build

FROM alpine:3.20

COPY --from=builder /app/dist /dist

CMD ["sh", "-c", "cp -r /dist/. /output/"]
