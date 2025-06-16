fn main() {
    let target = std::env::var("TARGET").expect("No TARGET env variable set");
    let profile = std::env::var("PROFILE").expect("No PROFILE env variable set");
    // inject emscripten build options
    if target.contains("emscripten") {
        // debug options
        if profile == "debug" {
            println!("cargo::rustc-link-arg=-sASSERTIONS=2");
        }
        // compile options
        println!("cargo::rustc-link-arg=-fexceptions");
        println!("cargo::rustc-link-arg=-sUSE_PTHREADS=1");
        println!("cargo::rustc-link-arg=-sPTHREAD_POOL_SIZE=4");
        // memory options
        println!("cargo::rustc-link-arg=-sSTACK_SIZE=2MB");
        println!("cargo::rustc-link-arg=-sINITIAL_MEMORY=100MB");
        println!("cargo::rustc-link-arg=-sALLOW_MEMORY_GROWTH=1");
        println!("cargo::rustc-link-arg=-sMALLOC=mimalloc");
        // export options
        println!("cargo::rustc-link-arg=-sEXPORT_ES6=1");
        println!("cargo::rustc-link-arg=-sMODULARIZE");
        println!("cargo::rustc-link-arg=-sINVOKE_RUN=0");
        // exports
        println!("cargo::rustc-link-arg=--no-entry");
        let exports = [
            "UTF8ToString",
            "ccall",
            "_free_cstring",
            "_initialize_playground",
            "_shutdown_playground",
            "_start_playing",
            "_stop_playing",
            "_stop_playing_notes",
            "_set_volume",
            "_midi_note_on",
            "_midi_note_off",
            "_set_bpm",
            "_set_instrument",
            "_get_samples",
            "_get_example_scripts",
            "_get_quickstart_scripts",
            "_get_script_error",
            "_update_script",
        ];
        println!(
            "cargo::rustc-link-arg=-sEXPORTED_FUNCTIONS={}",
            exports.join(",")
        );
        // assets
        println!(
            "cargo::rustc-link-arg=--preload-file={}/assets@/assets",
            std::env::var("CARGO_MANIFEST_DIR").unwrap()
        );
    } else {
        println!("cargo::warning=This example only works with target 'wasm32-unknown-emscripten'")
    }
}
