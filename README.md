# RNovEmail

RNovEmail is a Rust-only internal mail service for organizations that need controlled mailbox assignment, provider-backed delivery, provider webhook intake, API automation, and Docker/Web deployment.

It embeds `RNovModularDB` as pinned Cargo git dependencies and stores service data in a local `.rnmdb` file. Self-registration is intentionally unsupported; users and mailboxes are assigned through admin/API flows.

This repository keeps production code and visitor-facing documentation only. Local plans, tests, test assets, and build outputs are ignored.

## Run

Set the required environment values from `.env.example`, then run the service:

```bash
cargo run -p rnovemail-bin
```

The default bind address is `127.0.0.1:18089`.

Build the container image:

```bash
docker build -t rnovemail:local .
```

Run with Docker Compose:

```bash
mkdir -p secrets
openssl rand -base64 32 > secrets/rnovemail_master_key
RNOVEMAIL_BOOTSTRAP_ADMIN_TOKEN=change-me docker compose up --build
```
