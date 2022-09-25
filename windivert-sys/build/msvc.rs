use std::{env, fs};

use cc::Build;

const MSVC_ARGS: &str = r#"/Ivendor\include /ZI /JMC /nologo /W1 /WX- /diagnostics:column /O1 /Oi /D WIN32 /D NDEBUG /D _WINDOWS /D _USRDLL /D DLL_EXPORTS /D _WINDLL /Gm- /EHsc /MDd /GS- /fp:precise /Zc:wchar_t /Zc:forScope /Zc:inline /Gd /TC /FC /errorReport:queue vendor\dll\windivert.c /link /ERRORREPORT:QUEUE /INCREMENTAL /NOLOGO kernel32.lib advapi32.lib /NODEFAULTLIB /DEF:vendor/dll/windivert.def /MANIFEST /manifest:embed /DEBUG:FASTLINK /TLDLIB:1 /ENTRY:"WinDivertDllEntry" /DYNAMICBASE /NXCOMPAT /DLL"#;
pub fn compile(build: Build) {
    let compiler = build.get_compiler();

    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search={}", &out_dir);

    let mut cmd = compiler.to_command();

    cmd.arg(format!(r#"/Fo{out_dir}\WinDivert.obj"#));
    cmd.arg(format!(r#"/Fd{out_dir}\WinDivert.pdb"#));

    cmd.args(MSVC_ARGS.split(" "));

    let arch = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_ref() {
        "x86" => "x86",
        "x86_64" => "x64",
        _ => panic!("Unsupported target architecture!"),
    };
    cmd.arg(format!("/MACHINE:{}", arch));

    cmd.arg(format!(r#"/PDB:{out_dir}\WinDivertDll.pdb"#));
    cmd.arg(format!(r#"/OUT:{out_dir}\WinDivert.dll"#));
    cmd.arg(format!(r#"/IMPLIB:{out_dir}\WinDivert.lib"#));

    if let Ok(out) = cmd.output() {
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

    if let Ok(dylib_save_dir) = env::var("WINDIVERT_DLL_OUTPUT") {
        let _ = fs::copy(
            format!(r#"{out_dir}\WinDivert.dll"#),
            format!(r#"{dylib_save_dir}\WinDivert.dll"#),
        );
        let _ = fs::copy(
            format!(r#"{out_dir}\WinDivert.lib"#),
            format!(r#"{dylib_save_dir}\WinDivert.lib"#),
        );
    } else {
        println!("cargo:warning=Environment variable WINDIVERT_DLL_OUTPUT not found, compiled dll & lib files will be stored on {out_dir}");
    };
}
