# see https://github.com/casey/just

default: build

build:
    cargo build --release

run:
    cargo run --release --example=play-script --features=player

docs-generate-api:
    cd docs/generate && cargo run -- "../../types/nerdo/library" "../src"

docs-build: docs-generate-api
    cd docs && mdbook build
    
docs-serve: docs-generate-api
    cd docs && mdbook serve
