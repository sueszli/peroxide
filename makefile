.PHONY: build
build:
	cargo install wasm-pack
	wasm-pack build --target web

.PHONY: up
up:
	git pull
	git add .
	if [ -z "$(msg)" ]; then git commit -m "up"; else git commit -m "$(msg)"; fi
	git push
