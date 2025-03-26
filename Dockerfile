FROM docker.io/clux/muslrust:stable as builder

WORKDIR /app

COPY src /app/src
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock
COPY modpacks /app/modpacks
COPY translations /app/translations
COPY translateutil /app/translateutil

CMD ["mkdir", "-p", "/app/target/normal", "/app/target/interactive/en", "/app/target/interactive/es"]

CMD ["cargo", "build", "--release", "--features", "interactive es", "--target", "x86_64-unknown-linux-musl"]
#CMD ["cargo", "build", "--release", "--features", "interactive", "--target-dir", "/app/target/interactive/en", "--target", "x86_64-unknown-linux-musl"]
#CMD ["cargo", "build", "--release", "--features", "interactive es", "--target-dir", "/app/target/interactive/es", "--target", "x86_64-unknown-linux-musl"]
