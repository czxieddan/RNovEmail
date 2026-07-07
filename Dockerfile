FROM rust:1.95-bookworm AS builder
ARG HTTP_PROXY
ARG HTTPS_PROXY
ARG ALL_PROXY
ARG NO_PROXY
ENV HTTP_PROXY=${HTTP_PROXY}
ENV HTTPS_PROXY=${HTTPS_PROXY}
ENV ALL_PROXY=${ALL_PROXY}
ENV NO_PROXY=${NO_PROXY}
ENV CARGO_HTTP_MULTIPLEXING=false
ENV CARGO_HTTP_TIMEOUT=600
ENV CARGO_NET_RETRY=10
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release -p rnovemail-bin

FROM debian:bookworm-slim
RUN useradd --system --uid 10001 --home /nonexistent --shell /usr/sbin/nologin rnovemail \
    && mkdir -p /var/lib/rnovemail \
    && chown -R rnovemail:rnovemail /var/lib/rnovemail
WORKDIR /app
COPY --from=builder /app/target/release/rnovemail /usr/local/bin/rnovemail
USER 10001
EXPOSE 18089
ENTRYPOINT ["/usr/local/bin/rnovemail"]
