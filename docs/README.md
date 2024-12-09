## AFSEQ Book

### Requirements

The docs are generated using [mdBook](https://github.com/rust-lang/mdBook). To preview the pages locally you will need [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) to install mdbook, mdbook-linkcheck and mdbook-toc.

```sh
cargo install mdbook mdbook-linkcheck mdbook-toc
```

### Building

Afterwards you can serve the docs at `localhost:3000` using mdbook, this will automatically refresh the browser tab whenever you change markdown files.

```sh
mdbook serve --open
```

### Generate API reference

See [generate/README.md](./generate/README.md)