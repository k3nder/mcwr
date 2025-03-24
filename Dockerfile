FROM docker.io/clux/muslrust:stable as builder

WORKDIR /app

COPY src /app/src
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock
COPY modpacks /app/modpacks

CMD ["cargo", "build", "--release", "--target", "x86_64-unknown-linux-musl"]
