FROM rust:1.76.0
WORKDIR /app-backend
COPY . .
RUN cargo install --path ./web-server
RUN cargo build --release
ENTRYPOINT ["cargo","run","--release"]