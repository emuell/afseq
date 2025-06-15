use luals_docs_gen::*;

fn main() -> Result<(), Error> {
    // run from the `generate` dir
    std::env::set_current_dir(env!("CARGO_MANIFEST_DIR"))?;
    // set option and generate...
    let options = Options {
        library: "../../types/pattrns/library".into(),
        output: "../src".into(),
        excluded_classes: ["TimeContext", "TriggerContext", "pulse", "parameter"]
            .into_iter()
            .map(String::from)
            .collect(),
        order: OutputOrder::ByFile,
        namespace: "".into(),
    };
    generate_docs(&options)
}
