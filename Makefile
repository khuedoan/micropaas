.POSIX:
.PHONY: dev fmt lint test

.radicle:
	RAD_HOME=$(shell pwd)/.radicle rad auth

dev: .radicle
	docker compose up --build --watch

fmt:
	cargo fmt

lint:
	cargo clippy -- --deny warnings

test:
	cargo test
