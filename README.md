# Simple Webhook Rust Server

Very minimal webhook receiver built with Rust and [axum](https://github.com/tokio-rs/axum).

It exposes a single endpoint `/` that supports both:

- `POST /` – receive and store webhook payloads
- `GET /?secret=DEFAULT_KEY` – view stored webhook payloads

Mainly built for hosting on my Docker server to test webhooks, reason the very simplistic authentication.

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
