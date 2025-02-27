.POSIX:
.PHONY: dev fmt test

.radicle:
	RAD_HOME=$(shell pwd)/.radicle rad auth

dev: .radicle
	docker compose up --build --watch

fmt:
	cargo fmt

test:
	cargo test
