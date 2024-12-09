FROM rust:1-slim AS builder

WORKDIR /src

# Dummy source to cache dependencies
COPY Cargo.toml Cargo.lock .
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target "$(uname -m)-unknown-linux-gnu" \
    && rm -rf src

# Actual source code
COPY . .
RUN RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target "$(uname -m)-unknown-linux-gnu"
RUN cp "/src/target/$(uname -m)-unknown-linux-gnu/release/update" /usr/local/bin/update
RUN cp "/src/target/$(uname -m)-unknown-linux-gnu/release/post-receive" /usr/local/bin/post-receive

FROM nixos/nix:latest

COPY ./nix/nix.conf /etc/nix/nix.conf

RUN nix-env -iA \
    nixpkgs.docker \
    nixpkgs.git \
    nixpkgs.nixpacks \
    nixpkgs.soft-serve \
    nixpkgs.stagit \
    nixpkgs.yq-go

ENV SOFT_SERVE_DATA_PATH=/var/lib/micropaas
ENV SOFT_SERVE_HTTP_LISTEN_ADDR=:8080
ENV SOFT_SERVE_SSH_LISTEN_ADDR=:2222

WORKDIR ${SOFT_SERVE_DATA_PATH}

COPY ./stagit /etc/stagit

COPY --from=builder /usr/local/bin/update ./hooks/update
COPY --from=builder /usr/local/bin/post-receive ./hooks/post-receive

CMD [ "soft", "serve" ]
