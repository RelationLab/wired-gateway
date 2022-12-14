FROM lukemathwalker/cargo-chef:latest as chef
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin wired-gateway

FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get dist-upgrade -y \
    && apt-get install -y ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/wired-gateway wired-gateway
ENTRYPOINT ["./wired-gateway"]