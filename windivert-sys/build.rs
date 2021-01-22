use std::env;

fn main() {
    if let Err(_) = std::env::var("DOCS_RS") {
        println!("cargo:rerun-if-changed=wrapper.h");

        let lib_path = env::var("WINDIVERT_LIB")
            .expect("Te folder containing the dll, lib and sys must be specified using WINDIVERT_LIB env variable.");
        println!("cargo:rustc-link-search={}", &lib_path);

        // Link
        println!("cargo:rustc-link-lib=dylib=WinDivert");
    }
}
