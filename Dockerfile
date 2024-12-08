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
COPY ./hooks ./hooks

CMD [ "soft", "serve" ]
