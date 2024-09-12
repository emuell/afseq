use std::{
    fs::*,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use clap::Parser as ClapParser;

mod error;
mod json;
mod json_to_types;
mod library;
mod lua_parser;
mod render;
mod types;

use error::Error;
use library::Library;

#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    library: PathBuf,
    output: PathBuf,
}

#[derive(Clone, Debug)]
struct TocEntry {
    file_path: String,
    file_name: String,
    link: String,
}

impl TocEntry {
    fn from(name: &str) -> Self {
        let mut file_name = name.to_string();
        let mut display_name = name.to_string();
        let mut file_path = String::new();
        let mut level = String::from("  ");
        if name.contains('/') {
            let full_name = file_name.clone();
            let mut splits = full_name.split('/').collect::<Vec<_>>();
            level = "  ".repeat(splits.len());
            file_name = splits.remove(splits.len() - 1).to_string();
            display_name.clone_from(&file_name);
            file_path = splits
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
                .join("/");
            file_path.push('/');
        } else {
            display_name = match name {
                "builtins" => "Builtin Types".to_string(),
                "modules" => "Module Extensions".to_string(),
                "structs" => "Helper Types".to_string(),
                "global" => "Globals".to_string(),
                _ => name.to_string(),
            }
        }
        let link = format!(
            "{}- [{}](API/{}{}.md)",
            level, display_name, file_path, file_name
        )
        .to_string();
        Self {
            file_path,
            file_name,
            link,
        }
    }
}

fn replace_toc_in_file(file_path: &Path, toc: &[String]) -> Result<(), Error> {
    let mut file = File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    drop(file);

    let mut lines = content.lines().collect::<Vec<&str>>();
    let toc_start_line = lines
        .iter()
        .enumerate()
        .find(|(_i, l)| l.contains("<!-- API TOC START -->"))
        .expect("Failed to locate <API TOC START> line in Summary.md")
        .0;
    let toc_end_line = lines
        .iter()
        .enumerate()
        .find(|(_i, l)| l.contains("<!-- API TOC END -->"))
        .expect("Failed to locate <API TOC END> line in Summary.md")
        .0;
    lines.splice(
        (toc_start_line + 1)..toc_end_line,
        toc.iter().map(String::as_str),
    );

    let mut file = File::create(file_path)?;
    file.write_all(lines.join("\n").as_bytes())?;
    Ok(())
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let lib = Library::from_path(&args.library)?;
    let docs = lib.export_docs();

    // clear previously generated API doc files (except README.md)
    let api_path = args.output.clone().join("API");
    for entry in read_dir(&api_path)?.flatten() {
        if entry.path().is_dir() {
            remove_dir_all(entry.path())?;
        } else if entry
            .path()
            .file_name()
            .is_some_and(|file| !file.eq_ignore_ascii_case("README.md"))
        {
            remove_file(entry.path())?;
        }
    }

    // write all documents from the library to a file using their names and create an API TOC
    let mut toc_links = vec![];
    for (name, content) in &docs {
        let toc_entry = TocEntry::from(name);
        toc_links.push(toc_entry.link);
        let dir_path = api_path.clone().join(toc_entry.file_path.clone());
        let file_path = dir_path.clone().join(toc_entry.file_name + ".md");
        if !dir_path.exists() {
            create_dir(dir_path)?;
        }
        println!("Creating '{}'", file_path.to_string_lossy());
        let mut file = File::create(file_path)?;
        file.write_all(content.as_bytes())?;
    }
    replace_toc_in_file(&args.output.clone().join("SUMMARY.md"), &toc_links)?;
    Ok(())
}
