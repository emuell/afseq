# AFSEQ API Documentation Generator

This app generates the API definition chapters in the [AFSEQ book](https://emuell.github.io/afseq/) from the [Lua API definition](../../types/nerdo/) files using [luals-docs-gen](https://github.com/emuell/luals-docs-gen).

## Requirements

[Rust](https://www.rust-lang.org/tools/install) v1.78 or higher

## Building

To create or update the API definitions chapter, build and run the app, then build the book:

```bash
# in the afseq root directory
cd docs 
# build and run the generate app to create the API definition
cargo run
# build or serve the book with the updated API definition
mdbook serve
```

---

Alternatively, if you have vscode installed, open the afseq `./docs` folder and use the provided build task to build the API and the book:

- `build: API Docs`: compiles and runs the API docs generator
- `build: book`: compiles the mdbook
- `serve: book`: serve and live update the book at //localhost:3000 
