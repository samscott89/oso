.PHONY: build clean test nodejs bundler web

CARGO_FLAGS ?= --dev

build: nodejs

clean:
	@rm -rf pkg

nodejs web: clean
	wasm-pack -q build $(CARGO_FLAGS) --target $@ -d pkg
	@rm pkg/.gitignore pkg/package.json
	@cp pkg/* ../vscode/oso-lsp/server/src/
	@mkdir -p ../vscode/oso-lsp/server/dist/src
	@cp pkg/* ../vscode/oso-lsp/server/dist/src/

bundler: clean
	wasm-pack -q build $(CARGO_FLAGS) --target $@ -d pkg
	@rm pkg/.gitignore pkg/package.json
	@mkdir -p ../vscode/oso-lsp/server/dist/bundler
	@cp pkg/* ../vscode/oso-lsp/server/dist/bundler/

test:
	wasm-pack test --node
