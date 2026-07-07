# RNovEmail

RNovEmail is planned as a Rust-only internal mail service for organizations that need controlled mailbox assignment, provider-backed delivery, provider webhook intake, API automation, and Docker/Web deployment.

The service will embed `RNovModularDB` as a pinned Cargo git dependency. Self-registration is intentionally unsupported; users and mailboxes are assigned through the admin surface or API.

This repository keeps production code and visitor-facing documentation only. Local plans, test code, test assets, and build outputs are ignored.

## Run

Set the required environment values from `.env.example`, then run the service:

```bash
cargo run -p rnovemail-bin
```

Build the container image:

```bash
docker build -t rnovemail:local .
```
