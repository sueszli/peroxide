.PHONY: build
build:
	wasm-pack build --target web
	live-server --port=8080 --entry-file=index.html

.PHONY: build-release
build-release:
	wasm-pack build --release --target web
	rm pkg/.gitignore

.PHONY: fmt
fmt:
	cargo fmt -- --config max_width=5000
