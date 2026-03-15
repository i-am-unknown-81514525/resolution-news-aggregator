FROM rust:1.93-slim as planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.93-slim as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.93-slim as builder
WORKDIR /app
COPY . .
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get install -y libc6-amd64 libc6-dev libc6-dbg
COPY --from=builder /app/target/release/news-aggregator /app/news-agg
CMD ["./news-agg"]