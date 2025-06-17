# Examples

## `Web Examples`

### `playground`

This is a simple WASM online playground. It's hosted here: https://pattrns.renoise.com/

See [playground's README](./playground/README.md) on how to run and build the example locally.

## `Rust Examples`

### `play.rs`

This only uses the pattrns rust lib. It defines and plays a little music thing. If you change the content here, you will need to recompile and restart the example.

### `play-script.rs`

This uses the pattrns [Lua API](../types/pattrns/). It also defines and plays a little music thing, but [its contents](./assets/) can be added/removed and changed on the fly, so you can do some basic live music coding here.  

#### Running

CD into the main folder, the folder where the `Cargo.toml` file is. 

Then use the following commands to compile and run the examples:

```bash
# play.rs
cargo run --release --example=play --features=player,cpal-output

# play-script.rs
cargo run --release --example=play-script --features=player,cpal-output
```

Aternatively you can also open the pattrns root folder with vscode and use the `play` or `play-script` launch tasks.

#### Requirements

[Rust toolchain](https://www.rust-lang.org/tools/install) with Rust edition >= 2021.
