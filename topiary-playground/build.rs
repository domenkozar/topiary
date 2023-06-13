use itertools::Itertools;
use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env::current_dir, fs};

use topiary::{Configuration, SupportedLanguage};

fn to_js_string(path: PathBuf) -> String {
    fs::read_to_string(path)
        .unwrap()
        // Escape the escape char itself. Must be done first.
        .replace('\\', "\\\\")
        // Backticks are delimiters for template literals in JS.
        .replace('`', "\\`")
        // Template literals interpolate `${foo}` as the value of foo.
        // Escape "${" to prevent this.
        .replace("${", "\\${")
}

fn main() {
    println!("cargo:rerun-if-changed=../languages/");
    println!("cargo:rerun-if-changed=../topiary/tests/samples/input/");

    // Export test samples and queries as JS files
    let language_dir = current_dir().unwrap().join("../languages/");
    let language_files = fs::read_dir(language_dir).unwrap();

    let mut language_map: HashMap<String, String> = HashMap::new();
    for path in language_files {
        let path = path.unwrap().path();
        if let Some(ext) = path.extension().map(|ext| ext.to_string_lossy()) {
            if ext != "scm" {
                continue;
            }

            let name: String = path.file_stem().unwrap().to_string_lossy().to_string();

            let content = to_js_string(path);
            language_map.insert(name, content);
        }
    }

    let input_dir = current_dir()
        .unwrap()
        .join("../topiary/tests/samples/input/");
    let input_files = fs::read_dir(input_dir).unwrap();

    let mut input_map: HashMap<String, String> = HashMap::new();
    for path in input_files {
        let path = path.unwrap().path();
        if let Some(ext) = path.extension().map(|ext| ext.to_string_lossy()) {
            if !Configuration::parse_default_configuration()
                .known_extensions()
                .contains(&*ext)
                || ext == "mli"
            {
                // skip ocaml.mli, keep ocaml.ml
                continue;
            }

            let prefix: String = path.file_stem().unwrap().to_string_lossy().to_string();
            let content = to_js_string(path);
            input_map.insert(prefix, content);
        }
    }

    let mut buffer = String::new();

    for (name, query) in language_map.into_iter().sorted() {
        if let Some(input) = input_map.get(&name) {
            let supported = SupportedLanguage::is_supported(&name);

            buffer.push_str(&format!(
                r#"  "{name}": {{
    supported: `{supported}`,
    query: `{query}`,
    input: `{input}`,
  }},
"#
            ));
        }
    }

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("languages_export.ts");
    let mut js_export = fs::File::create(dest_path).unwrap();
    write!(
        js_export,
        r#"// This file is automatically generated by Cargo.
// It is not intended for manual editing.
// To generate, please run `cargo build -p topiary-playground`
const languages: {{[index: string]: any}} = {{
{buffer}}};

export default languages;
"#
    )
    .unwrap();
}
