.POSIX:
.PHONY: dev fmt

dev:
	docker compose up --watch

fmt:
	cargo fmt
