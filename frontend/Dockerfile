FROM oven/bun:1 as base
WORKDIR /app

COPY . .

RUN bun i
RUN bun run build
CMD bun run preview --host
