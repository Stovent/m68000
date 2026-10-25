use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-changed=cbindgen-ffi.toml");
    println!("cargo::rerun-if-changed=c_bindings.c");

    let out_dir = env::var("OUT_DIR").unwrap();
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let output_file = PathBuf::from(&crate_dir).join("../include/m68000/m68000-ffi.h");

    let config = cbindgen::Config::from_file("cbindgen-ffi.toml").expect("cbindgen-ffi.toml not found");

    // Generate the C header
    cbindgen::Builder::new()
        .with_crate(crate_dir.clone())
        .with_config(config)
        .with_language(cbindgen::Language::C)
        .generate()
        .expect("Failed to generate C bindings")
        .write_to_file(output_file);

    // Check that it compiles.
    // Find our C compiler.
    let build = cc::Build::new();
    let compiler = build.get_compiler();
    let cc_path = compiler.path();

    let mut cmd = std::process::Command::new(cc_path);

    if compiler.is_like_msvc() || compiler.is_like_clang_cl() {
        cmd.arg("/c")
           .arg("c_bindings.c")
           .arg(format!("/Fo:{}/c_bindings.obj", out_dir));
    } else {
        cmd.arg("-c")
           .arg("c_bindings.c")
           .arg("-o")
           .arg(format!("{}/c_bindings.o", out_dir))
           .arg("-Wall")
           .arg("-Wextra");
    }

    let output = cmd.output().expect("Failed to run C compiler");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "Failed to compile C file.\nSTDOUT:\n{}\nSTDERR:\n{}",
            stdout, stderr
        );
    }
}
