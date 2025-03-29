extern crate proc_macro;

use c2rust_transpile::RustEdition;
use proc_macro::TokenStream;
use std::path::{Path, PathBuf};

use anyhow::Context;
use rand::Rng;

use c2rust_transpile::{transpile_single_to_buffer, CompileCmd, ReplaceMode, TranspilerConfig};

#[proc_macro]
pub fn c(input: TokenStream) -> TokenStream {
    let c_code = input.to_string();

    // need to detect something that has runtim different between rust  Edition2021 Edition2024

    // don't want to add syn yet
    assert!(
        c_code.starts_with("r#\""),
        "C code must be passed as rust raw string literal starting with r#\""
    );
    assert!(
        c_code.ends_with("\"#"),
        "C code must be passed as rust raw string literal"
    );

    let transplied_code =
        transpile_code(&c_code[3..c_code.len() - 2]).expect("transpilation failure");

    transplied_code.parse().expect("rust parse output error")
}

fn temp_build_dir() -> std::io::Result<PathBuf> {
    let tmp = std::env::temp_dir();
    let random: u64 = rand::rng().random();

    let build_path = tmp.join("include-c2rust").join(random.to_string());
    std::fs::create_dir_all(&build_path)?;
    Ok(build_path)
}

fn transpile_code(c_code: &str) -> anyhow::Result<String> {
    let build_dir = temp_build_dir().context("unable to make tmp dir")?;
    eprintln!("output dir: {}", build_dir.display());

    let c_file_path = build_dir.join("code.c");
    std::fs::write(&c_file_path, c_code).expect("Unable to write c code to temporary file.");
    let transpile_config = make_transpile_config();
    let cc_db = compile_commands(&build_dir, &c_file_path);
    eprintln!("ccdb {cc_db:?}");

    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = LOCK.lock().unwrap();
    // something this function calls isn't re-entrant
    // add a mutex guard to prevent it from causing flaky tests
    // TODO remove this and fix the code at the location below
    //
    // ast_exporter: for the -p option: may only occur zero or one times!
    // [Parse Error]
    // include-d771570fb872ba35: c2rust-ast-exporter/src/AstExporter.cpp:2815: Outputs process(int, const char**, int*): Assertion `0 && "Failed to parse command line options"' failed.
    // error: test failed, to rerun pass `--lib`

    let (out, _, _) = transpile_single_to_buffer(&transpile_config, c_file_path, &cc_db, &[])
        .map_err(|()| std::io::Error::other("transpilation failure"))?;

    Ok(out)
}

fn compile_commands(build_dir: &Path, source_file: &Path) -> PathBuf {
    let cc_db_file = build_dir.join("compile_commands.json");

    let absolute_path = std::fs::canonicalize(source_file)
        .unwrap_or_else(|_| panic!("Could not canonicalize {}", source_file.display()));

    let compile_commands = [CompileCmd {
        directory: PathBuf::from("."),
        file: absolute_path.clone(),
        arguments: vec![
            "clang".to_string(),
            absolute_path.to_str().unwrap().to_owned(),
        ],
        command: None,
        output: None,
    }];

    let json_content = serde_json::to_string(&compile_commands).unwrap();
    std::fs::write(&cc_db_file, &json_content)
        .expect("Failed to write to temporary compile_commands.json");

    cc_db_file
}

fn make_transpile_config() -> TranspilerConfig {
    // note: lots of massaging had to be done with these args to get things to work
    // don't expect things to work properly if you change the options
    TranspilerConfig {
        verbose: true,
        incremental_relooper: true,
        fail_on_multiple: true,
        use_c_multiple_info: true,
        simplify_structures: true,
        panic_on_translator_failure: true,
        emit_modules: true, // must be true, otherwise we emit #![allow(...)] attributes in the middle of a file
        fail_on_error: true,
        replace_unsupported_decls: ReplaceMode::None, // not sure about this one
        translate_valist: false,                      // must be false, otherwise we require nightly
        overwrite_existing: true,
        reduce_type_annotations: true,
        reorganize_definitions: false,
        emit_no_std: false,
        output_dir: None, // broken if we specify output dir
        disable_refactoring: false,
        preserve_unused_functions: true,
        // below are things we don't care about
        dump_untyped_context: false,
        dump_typed_context: false,
        pretty_typed_context: false,
        dump_function_cfgs: false,
        json_function_cfgs: false,
        dump_cfg_liveness: false,
        dump_structures: false,
        debug_ast_exporter: false,
        filter: None,
        debug_relooper_labels: false,
        prefix_function_names: None,
        translate_asm: false,
        use_c_loop_info: false,
        enabled_warnings: std::collections::HashSet::new(),
        translate_const_macros: false,
        translate_fn_macros: false,
        log_level: None,
        max_rust_edition: RustEdition::Edition2024,
        emit_build_files: false,
        binaries: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile() {
        let rust_translation = transpile_code("int foo() {return 1;}").unwrap();

        insta::assert_snapshot!(rust_translation, @r###"
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn foo() -> core::ffi::c_int {
            unsafe {
                return 1 as core::ffi::c_int;
            }
        }
        "###);
    }

    #[test]
    fn test_compile_access_ptr() {
        let rust_translation = transpile_code("int access_ptr(int* p) { return *p; }").unwrap();

        insta::assert_snapshot!(rust_translation, @r###"
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn access_ptr(mut p: *mut core::ffi::c_int) -> core::ffi::c_int {
        unsafe {
            return *p;
        }
    }
    "###);
    }
}
