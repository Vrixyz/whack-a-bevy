FROM rust:1.58 as builder
WORKDIR "/usr/home/app"
COPY ./crates ./crates
RUN rm -rf ./crates/example_client*
COPY ./Cargo.toml .
COPY ./Cargo.lock .
RUN cargo build --release --bin example_server
FROM debian:buster-slim
RUN apt-get update -y  && apt-get install -y g++ pkg-config libx11-dev libasound2-dev alsa-utils libudev-dev
ARG PORT
WORKDIR "/usr/home/app"
COPY --from=builder /usr/home/app ./
CMD ./target/release/example_server
