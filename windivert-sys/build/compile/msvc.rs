use cc::Build;
use path_macro2::{path, path_const};
use std::{env, fs};

use crate::DLL_OUTPUT_PATH_ARG;

pub fn lib() {
    let mut build = Build::new();
    let out_dir = env::var("OUT_DIR").unwrap();

    build
        .out_dir(&out_dir)
        .include(path_const!(vendor / include))
        .file(path_const!(vendor / dll / windivert.c));

    for &flag in STATIC_CL_ARGS {
        build.flag(flag);
    }
    build.compile("WinDivert");
}

pub fn dll() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut compiler = Build::new().get_compiler().to_command();

    let arch = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_ref() {
        "x86" => "x86",
        "x86_64" => "x64",
        _ => panic!("Unsupported target architecture!"),
    };

    for &flag in DYNAMIC_CL_ARGS {
        compiler.arg(flag);
    }

    compiler.arg(&format!("/MACHINE:{arch}"));

    compiler.arg(&format!(
        "/PDB:{}",
        &path!({ out_dir } / WinDivertDll.pdb).to_string_lossy()
    ));
    compiler.arg(&format!(
        "/OUT:{}",
        &path!({ out_dir } / WinDivert.dll).to_string_lossy()
    ));
    compiler.arg(&format!(
        "/IMPLIB:{}",
        &path!({ out_dir } / WinDivert.lib).to_string_lossy()
    ));

    if let Ok(out) = compiler.output() {
        if !out.status.success() {
            panic!(
                "\nERROR: {:?}\n{}\n",
                &out.status,
                String::from_utf8_lossy(&out.stdout),
            )
        }
    } else {
        panic!("Error compiling windivert dll.");
    }

    if let Ok(dylib_save_dir) = env::var(DLL_OUTPUT_PATH_ARG) {
        let _ = fs::copy(
            path!({ out_dir } / WinDivert.dll),
            path!({ dylib_save_dir } / WinDivert.dll),
        );
        let _ = fs::copy(
            path!({ out_dir } / WinDivert.lib),
            path!({ dylib_save_dir } / WinDivert.lib),
        );
    } else {
        println!("cargo:warning=Environment variable {DLL_OUTPUT_PATH_ARG} not found, compiled dll & lib files will be stored on {out_dir}");
    };
}

const DYNAMIC_CL_ARGS: &[&str] = &[
    concat!("/I", path_const!(vendor / include)),
    r#"/ZI"#,
    r#"/JMC"#,
    r#"/nologo"#,
    r#"/W1"#,
    r#"/WX-"#,
    r#"/diagnostics:column"#,
    r#"/O1"#,
    r#"/Oi"#,
    r#"/Gm-"#,
    r#"/EHsc"#,
    r#"/MDd"#,
    r#"/GS-"#,
    r#"/fp:precise"#,
    r#"/Zc:wchar_t"#,
    r#"/Zc:forScope"#,
    r#"/Zc:inline"#,
    r#"/Gd"#,
    r#"/TC"#,
    r#"/FC"#,
    r#"/errorReport:queue"#,
    path_const!(vendor / dll / windivert.c),
    r#"/link"#,
    r#"/ERRORREPORT:QUEUE"#,
    r#"/INCREMENTAL"#,
    r#"/NOLOGO"#,
    r#"kernel32.lib"#,
    r#"advapi32.lib"#,
    r#"/NODEFAULTLIB"#,
    concat!("/DEF:", path_const!(vendor / dll / windivert.def)),
    r#"/MANIFEST"#,
    r#"/manifest:embed"#,
    r#"/DEBUG:FASTLINK"#,
    r#"/TLDLIB:1"#,
    r#"/ENTRY:"WinDivertDllEntry""#,
    r#"/DYNAMICBASE"#,
    r#"/NXCOMPAT"#,
    r#"/DLL"#,
];

const STATIC_CL_ARGS: &[&str] = &[
    r#"/nologo"#,
    r#"/WX-"#,
    r#"/diagnostics:column"#,
    r#"/O1"#,
    r#"/Oi"#,
    r#"/EHsc"#,
    r#"/GS-"#,
    r#"/fp:precise"#,
    r#"/Zc:wchar_t"#,
    r#"/Zc:forScope"#,
    r#"/Zc:inline"#,
    r#"/Gd"#,
    r#"/TC"#,
    r#"/FC"#,
    r#"/errorReport:queue"#,
];
