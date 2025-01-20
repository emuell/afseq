use std::path::PathBuf;

use luals_docs_gen::*;

fn main() -> Result<(), Error> {
    // run from the `generate` dir
    std::env::set_current_dir(env!("CARGO_MANIFEST_DIR"))?;
    // set option and generate...
    let options = Options {
        library: PathBuf::from("../../types/nerdo/library"),
        output: PathBuf::from("../src"),
    };
    generate_docs(&options)
}
