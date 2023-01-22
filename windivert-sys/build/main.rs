mod gnu;
mod msvc;

use std::{env, fs};

use cc::Build;

fn main() {
    if std::env::var("DOCS_RS").is_err() {
        let out_dir = env::var("OUT_DIR").unwrap();
        println!("cargo:rerun-if-env-changed=WINDIVERT_PATH");
        println!("cargo:rerun-if-env-changed=WINDIVERT_DLL_OUTPUT");
        println!("cargo:rerun-if-env-changed={out_dir}/WinDivert.dll");
        println!("cargo:rerun-if-env-changed={out_dir}/WinDivert.lib");

        println!("cargo:rustc-link-lib=dylib=WinDivert");
        println!("cargo:rustc-link-search=native={out_dir}");

        if let Ok(lib_path) = env::var("WINDIVERT_PATH") {
            println!("cargo:warning=Copying windivert dll, lib & sys files from the path provided if present.");
            for f in fs::read_dir(&lib_path).unwrap() {
                let file = f.unwrap();
                if let Some(name) = file.file_name().to_str() {
                    match name {
                        "WinDivert.dll" | "WinDivert.lib" | "WinDivert32.sys"
                        | "WinDivert64.sys" => {
                            let _ = fs::copy(file.path(), format!("{out_dir}/{name}"));
                        }
                        _ => {}
                    }
                }
            }
        } else if cfg!(feature = "vendored") {
            println!("cargo:rerun-if-changed=wrapper.h");
            println!("cargo:rustc-link-search=native={out_dir}");
            println!("cargo:warning=Environment variable WINDIVERT_PATH not found, building WinDivert from source.");
            build_windivert();
        } else {
            panic!("Environment variable WINDIVERT_PATH not found and feature vendored not enabled, please provide the path to the WinDivert library files or enable the vendored feature to compile from source.");
        };

        let arch = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_ref() {
            "x86" => "32",
            "x86_64" => "64",
            _ => panic!("Unsupported target architecture!"),
        };

        if let Err(_) = fs::metadata(format!("{out_dir}\\WinDivert{arch}.sys")) {
            println!(
                "cargo:warning=WinDivert{arch}.sys not found on the same directory as the dll."
            )
        }
    }
}

fn build_windivert() {
    let build = Build::new();
    let compiler = build.get_compiler();

    if compiler.is_like_msvc() {
        msvc::compile(build);
    } else if compiler.is_like_gnu() {
        if !env::var("TARGET").unwrap().contains("windows") {
            panic!("This library only works for windows targets")
        }
        gnu::compile(build);
    }
}
