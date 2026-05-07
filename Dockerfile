FROM rust:1.92.0-trixie as builder

RUN apt-get update && apt-get install -y \
    curl unzip clang pkg-config libssl-dev build-essential \
    gcc-aarch64-linux-gnu g++-aarch64-linux-gnu \
    && rm -rf /var/lib/apt/lists/*

RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install -y libssl-dev:arm64

ENV NVM_DIR=/root/.nvm
RUN mkdir -p $NVM_DIR

# Install NVM and Node 24
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.4/install.sh | bash \
    && . "$NVM_DIR/nvm.sh" \
    && nvm install 24 \
    && nvm use 24 \
    && nvm alias default 24

# Dynamically set PATH so it doesn't break on minor version updates
ENV PATH="/root/.nvm/versions/node/v24.15.0/bin:$PATH"

RUN curl -fsSL https://bun.sh/install | bash
ENV PATH="/root/.bun/bin:$PATH"

RUN node -v && npm -v && bun -v

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-leptos -y
RUN rustup target add wasm32-unknown-unknown
RUN rustup target add aarch64-unknown-linux-gnu
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ \
    PKG_CONFIG_ALLOW_CROSS=1 \
    LEPTOS_BIN_TARGET_TRIPLE=aarch64-unknown-linux-gnu

WORKDIR /app

COPY package.json bun.lockb* ./
RUN bun install

COPY . .

ENV RUST_BACKTRACE=full
RUN cargo leptos build --release -v

FROM --platform=linux/arm64 debian:trixie-slim as runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    openssl ca-certificates libgcc-s1 \
    && apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/aarch64-unknown-linux-gnu/release/server /app/chara
COPY --from=builder /app/target/site /app/site

RUN echo | ls -R

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT="site"
EXPOSE 3000

CMD ["/app/chara"]
