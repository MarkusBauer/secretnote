SecretNote
==========

SecretNote is a website where users can store short, secret messages. 
These messages are end-to-end encrypted and can be read only once. 
In addition, users can also have end-to-end encrypted chats.

SecretNote is free and open source software, powered by Rust, Redis, Angular and Typescript.
It was mainly developed to learn some Rust basics.

Website: [https://secretnote.mk-bauer.de](https://secretnote.mk-bauer.de)


Command Line Interface
----------------------
We provide a command line utility to store and retrieve notes (see release assets). Usage:
- `echo Message | secretnote-cli` (store a note, return two links. First link is the secret link, second link is the admin link.)
- `secretnote-cli 'https://secretnote.mk-bauer.de/note/<abc>#<def>'` (retrieve a note previously stored)

For self-hosted instances, use `--host <your-server>`.


Host your own instance
----------------------
Either download a release bundle, or use our Docker images. In any case you need a Redis server.

Docker example: `docker run --name secretnote -p 8080:8080 markusbauer/secretnote --redis 1.2.3.4`

For configuration SecretNote accepts commandline parameters or environment variables:

- `--bind 127.0.0.1:8080` / `SECRETNOTE_BIND=...` Address and port to listen on
- `--redis 127.0.0.1` / `SECRETNOTE_REDIS=...` Redis server address/port
- `--redis-db 0` / `SECRETNOTE_REDIS_DB=...` Redis server database number
- `--redis-auth <...>` / `SECRETNOTE_REDIS_AUTH=...` Redis server AUTH password (optional)
- `--threads <number of cpus>` / `SECRETNOTE_THREADS=...` Number of worker threads to use
- `--base-url` / `SECRETNOTE_BASE_URL` The base URL of this service, mainly to configure Telegram Bots 
- `--telegram-token <token>` / `SECRETNOTE_TELEGRAM_TOKEN=...` Token for the Telegram Bot

Example configuration for docker-compose: 
```
---
version: "2.1"
services:
  redis:
    image: redis:alpine
    volumes:
      - ./redis-data:/data
  secretnote:
    image: markusbauer/secretnote
    ports:
      - 8080:127.0.0.1:8080  # use a TLS frontend for this port
    depends_on: [redis]
```


For developers
--------------
Building the secretnote server is a two-step process. You need Rust and a recent version of NodeJS.
Check out the build scripts for details.

**Installing dependencies:** `cargo fetch  && cd secretnote-fe &&  npm install`

**Building the Rust server:**
`cargo build --release`

**Building the frontend:**
`npm run build`
