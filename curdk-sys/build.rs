use std::{env, path::Path, process::Command};

#[cfg(target_os = "linux")]
fn cdk5_config() -> Command {
    if let Some(path) = env::var_os("CDK5_CONFIG") {
        return Command::new(path);
    }

    let candidates = [
        Path::new("/usr/lib/x86_64-linux-gnu/libcdk5-dev/bin/cdk5-config"),
        Path::new("/usr/lib/aarch64-linux-gnu/libcdk5-dev/bin/cdk5-config"),
    ];
    candidates
        .iter()
        .find(|path| path.exists())
        .map(Command::new)
        .unwrap_or_else(|| Command::new("cdk5-config"))
}

#[cfg(target_os = "linux")]
fn compile() -> Vec<String> {
    println!("cargo:rustc-link-lib=dylib=ncurses");
    println!("cargo:rustc-link-lib=dylib=cdk");

    let output = cdk5_config()
        .arg("--cflags")
        .output()
        .unwrap_or_else(|error| panic!("Unable to execute cdk5-config: {error}"));
    if !output.status.success() {
        panic!(
            "cdk5-config --cflags failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    String::from_utf8(output.stdout)
        .expect("cdk5-config returned invalid UTF-8")
        .split_whitespace()
        .map(String::from)
        .collect()
}

#[cfg(not(target_os = "linux"))]
fn compile() -> Vec<String> {
    panic!("curdk-sys currently requires Linux and CDK5");
}

fn main() {
    println!("cargo:rerun-if-changed=src/wrapper.h");
    println!("cargo:rerun-if-env-changed=CDK5_CONFIG");
    bindgen::Builder::default()
        .header("src/wrapper.h")
        .clang_args(compile())
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(Path::new(&env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
