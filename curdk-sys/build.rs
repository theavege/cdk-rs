use std::{env, path::Path};

#[cfg(target_os = "linux")]
fn compile() -> Vec<String> {
    let library = "cdk";
    println!("cargo:rustc-link-lib=dylib=ncurses");
    println!("cargo:rustc-link-lib=dylib={library}");
    vec![String::from("-I/usr/include/cdk")]
}

fn main() {
    bindgen::Builder::default()
        .header("src/wrapper.h")
        .clang_args(compile())
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(Path::new(&env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
