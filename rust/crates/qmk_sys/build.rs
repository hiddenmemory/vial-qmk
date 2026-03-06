use bindgen::Formatter;
use serde::Deserialize;
use std::{env, path::PathBuf};

const HEADER_PATHS: &[&str] = &[
    "../../../quantum/quantum_keycodes.h",
    "../../../quantum/quantum.h",
    "../../../quantum/logging/print.h",
    "../../../quantum/action.h",
    "../../../quantum/painter/qp.h",
    "../../../quantum/keyboard.h",
    "../../../quantum/keymap_introspection.h",
    // "../../../quantum/logging/sendchar.h",
    "../../../quantum/rgb_matrix/rgb_matrix.h",
    "../../../quantum/split_common/transactions.h",
    // "../../../quantum/eeconfig.h",
    // "../../../quantum/keymap_extras/keymap_us.h",
    "../../../keyboards/mechboards/common/qp_font/pixellari18.qff.h",
    "../../../keyboards/mechboards/common/qp_font/pixellari24.qff.h",
    "../../../keyboards/mechboards/common/qp_images/solaire.qgf.h",
    "../../../keyboards/mechboards/common/qp_images/ChefSlime.qgf.h",
    "../../../keyboards/mechboards/common/qp_images/GarbageSlime.qgf.h",
    // This must be last
    "../../../keyboards/mechboards/lily58/r2g/config.h",
];

#[derive(Deserialize, Debug)]
struct CompilationUnit {
    file: String,
    arguments: Vec<String>,
}

fn main() {
    let contents = std::fs::read_to_string("../../../compile_commands.json")
        .expect("Unable to load the compile_commands.json");
    let units = serde_json::from_str::<Vec<CompilationUnit>>(&contents)
        .expect("Unable to parse compile_commands.json");
    let unit = units
        .iter()
        .find(|unit| unit.file.ends_with("/r2g.c"))
        .expect("Unable to find target unit");

    let mut include_next_argument = false;

    let filtered_unit_args = unit
        .arguments
        .iter()
        .filter_map(|argument| {
            let mut include = (argument.starts_with("-D") && !argument.starts_with("-D_")) // Define we want to keep
        || argument.starts_with("-I") // Include path
        || argument.starts_with("-c")
        || argument.starts_with("-f")
        || argument.starts_with("-std")
        || include_next_argument;

            include_next_argument = false;

            if argument.as_str().eq("-isystem") {
                include = true;
                include_next_argument = true;
            }

            if argument.as_str().eq("-include") {
                include = true;
                include_next_argument = true;
            }

            if include {
                if let Some(stripped_path) = argument.strip_prefix("-I") {
                    Some(format!("-I../../../{stripped_path}"))
                } else {
                    Some(argument.to_string())
                }
            } else {
                None
            }
        })
        .collect::<Vec<String>>();

    // bindgen will pass -D from BINDGEN_EXTRA_CLANG_ARGS to clang
    let cflags = std::env::var("BINDGEN_CFLAGS").unwrap_or_default();

    let extra_clang_args = cflags
        .split(" ")
        .filter(|s| s.starts_with("-D"))
        .collect::<Vec<&str>>()
        .join(" ");

    unsafe {
        env::set_var("BINDGEN_EXTRA_CLANG_ARGS", extra_clang_args);
    }

    let bindings = bindgen::builder()
        .headers(HEADER_PATHS.iter().map(|path| path.to_string()))
        .use_core()
        .clang_args(filtered_unit_args)
        .formatter(Formatter::Rustfmt)
        .rustified_enum(".*")
        .constified_enum_module(".*")
        .generate_comments(true);

    let bindings = bindings.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
