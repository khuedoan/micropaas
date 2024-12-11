.POSIX:
.PHONY: dev fmt test

dev:
	docker compose up --build --watch

fmt:
	cargo fmt

test:
	cargo test
