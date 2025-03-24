.PHONY: build
build:
	# cargo install wasm-pack
	wasm-pack build --target web

	# npm install -g live-server
	live-server --port=8080 --entry-file=index.html

.PHONY: up
up:
	git pull
	git add .
	if [ -z "$(msg)" ]; then git commit -m "up"; else git commit -m "$(msg)"; fi
	git push
