.POSIX:
.PHONY: dev fmt

dev:
	docker compose up --build --watch

fmt:
	cargo fmt
