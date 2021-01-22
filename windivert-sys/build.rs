use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let bindings = bindgen::Builder::default()
        .derive_default(true)
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .whitelist_type("P?WINDIVERT_(IP|IPV6|TCP)HDR")
        .whitelist_type("WINDIVERT_ADDRESS")
        .whitelist_type("WINDIVERT_DATA_.*")
        .whitelist_type("WINDIVERT_IOCTL")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("generated_bindings.rs"))
        .expect("Couldn't write bindings!");

    let lib_path = env::var("WINDIVERT_LIB")
        .expect("Te folder containing the dll, lib and sys must be specified using WINDIVERT_LIB env variable.");
    println!("cargo:rustc-link-search={}", &lib_path);

    // Link
    println!("cargo:rustc-link-lib=dylib=WinDivert");
}
