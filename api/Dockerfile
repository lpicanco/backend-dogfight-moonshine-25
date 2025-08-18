FROM rust:1.88-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*


WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY api/Cargo.toml ./api/
COPY processor/Cargo.toml ./processor/

# Create dummy source files for all workspace members
RUN mkdir -p api/src processor/src && \
    echo 'fn main() {}' > api/src/main.rs && \
    echo 'fn main() {}' > processor/src/main.rs && \
    echo 'pub fn dummy() {}' > processor/src/lib.rs

RUN cargo build --release --package moonshine-processor
RUN cargo build --release --package moonshine-api

COPY processor/src ./processor/src
COPY api/src ./api/src

RUN touch processor/src/lib.rs processor/src/main.rs
RUN cargo build --release --package moonshine-processor

RUN touch api/src/main.rs
RUN cargo build --release --package moonshine-api


FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/moonshine-api /usr/local/bin/

CMD ["moonshine-api"]