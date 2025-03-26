.PHONY: build
build:
	wasm-pack build --target web
	live-server --port=8080 --entry-file=index.html

.PHONY: build-release
build-release:
	wasm-pack build --release --target web
	rm pkg/.gitignore

.PHONY: up
up:
	git pull
	git add .
	if [ -z "$(msg)" ]; then git commit -m "up"; else git commit -m "$(msg)"; fi
	git push
