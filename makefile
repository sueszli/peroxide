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
	cargo fmt -- --config max_width=250

.PHONY: test
test:
	cargo test
	@echo "Running WASM tests for p2p.rs..."
	@if command -v chromedriver >/dev/null 2>&1 || command -v google-chrome >/dev/null 2>&1 || command -v chromium >/dev/null 2>&1; then \
		echo "Running tests with Chrome..."; \
		wasm-pack test --headless --chrome; \
	elif command -v geckodriver >/dev/null 2>&1 || command -v firefox >/dev/null 2>&1; then \
		echo "Chrome not found, trying Firefox..."; \
		wasm-pack test --headless --firefox; \
	else \
		echo "No supported browser found for headless testing"; \
		echo "Trying Chrome anyway (wasm-pack will download drivers if needed)..."; \
		wasm-pack test --headless --chrome; \
	fi
