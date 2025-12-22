# Simple Webhook Rust Server

Very minimal webhook receiver built with Rust and [axum](https://github.com/tokio-rs/axum).

It exposes a single endpoint `/` that supports both:

- `POST /` – receive and store webhook payloads
- `GET /?secret=<SECRET>` – view stored webhook payloads

Mainly built for hosting on my Docker server to test webhooks, reason the very simplistic authentication.

<img width="503" height="233" alt="000143 Zen 15 12 2025 04 07 09" src="https://github.com/user-attachments/assets/dde408b6-f0de-487a-a82d-d83ed48cc5ad" />

## Docker Compose

Example `docker-compose.yml`:

```yaml
services:
  simple-webhook-rust:
    restart: unless-stopped
    image: ghcr.io/bolado/simple-webhook-rust:master
    ports:
      - 3001:3001
    environment:
      - PORT=3001
      - WEBHOOK_SECRET=DEFAULT_KEY
```

You may have to adjust the network settings depending on your Docker setup. Also if you clone the repo and build the image yourself, make sure to change the image name accordingly.

If you omit `WEBHOOK_SECRET`, check the container or process logs for the generated secret line: `WEBHOOK_SECRET not set; generated secret: <SECRET>`.
