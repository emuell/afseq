fn main() {
    // inject emscripten build options for the playground example
    if std::env::var("TARGET").unwrap().contains("emscripten") {
        println!("cargo::rustc-link-arg=-fexceptions");
        println!("cargo::rustc-link-arg=--no-entry");
    }
}
