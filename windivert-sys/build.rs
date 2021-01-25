use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use cc::{Build, Tool};

fn print_env(compiler: &Tool) {
    eprintln!("Environment variables:");
    for (k, v) in env::vars() {
        eprintln!("{}={}", k, v);
    }
    eprintln!("\nCompiler:\n{}", compiler.path().to_string_lossy());

    eprintln!("\nCompiler environment variables:");
    for (k, v) in compiler.env().iter() {
        eprintln!("{}={}", k.to_string_lossy(), v.to_string_lossy());
    }
    eprintln!("");
}

fn main() {
    if let Err(_) = std::env::var("DOCS_RS") {
        println!("cargo:rerun-if-changed=wrapper.h");
        println!("cargo:rerun-if-env-changed=WINDIVERT_LIB");

        if let Ok(lib_path) = env::var("WINDIVERT_LIB") {
            println!("cargo:rustc-link-search={}", &lib_path);
        } else {
            build_windivert();
        }

        // Link
        println!("cargo:rustc-link-lib=dylib=WinDivert");
    }
}

fn build_windivert() {
    let build = Build::new();
    let compiler = build.get_compiler();
    print_env(&compiler);

    if compiler.is_like_msvc() {
        msvc_compile(build);
    } else if compiler.is_like_gnu() {
        if !env::var("TARGET").unwrap().contains("windows") {
            panic!("This library only works for windows targets")
        }
        gnu_compile(build);
    }
}

const MSVC_ARGS: &str = r#"/JMC /Ivendor\include /nologo /Zi /W1 /WX- /O1 /Oi /Oy- /D WIN32 /D _WINDOWS /D _USRDLL /D DLL_EXPORTS /D _WINDLL /Gm- /EHsc /MDd /GS- /fp:precise /Zc:wchar_t /Zc:forScope /Zc:inline /Gd /FC /TC /analyze- vendor\dll\windivert.c /link /NOLOGO kernel32.lib user32.lib gdi32.lib winspool.lib comdlg32.lib advapi32.lib shell32.lib ole32.lib oleaut32.lib uuid.lib odbc32.lib odbccp32.lib /NODEFAULTLIB /DEF:vendor/dll/windivert.def /MANIFEST /MANIFESTUAC:"level='asInvoker'" /MANIFESTUAC:"uiAccess='false'" /MANIFEST:EMBED /DEBUG:FASTLINK /ENTRY:WinDivertDllEntry /DYNAMICBASE /NXCOMPAT /DLL"#;
fn msvc_compile(build: Build) {
    let compiler = build.get_compiler();

    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search={}", &out_dir);

    let mut tmp_path = PathBuf::from(&out_dir);
    tmp_path.push("tmp-build");
    fs::create_dir(&tmp_path).expect("Unable to create temporary build folder");
    let tmp_dir = tmp_path.to_string_lossy();

    let mut cmd = compiler.to_command();

    cmd.arg(format!(r#"/Fo{}\WinDivert.obj"#, &tmp_dir));
    cmd.arg(format!(r#"/Fd{}\WinDivert.pdb"#, &tmp_dir));

    cmd.args(MSVC_ARGS.split(" "));

    cmd.arg(format!(r#"/PDB:{}\WinDivert.pdb"#, &tmp_dir));
    cmd.arg(format!(r#"/OUT:{}\WinDivert.dll"#, &tmp_dir));
    cmd.arg(format!(r#"/IMPLIB:{}\WinDivert.lib"#, &tmp_dir));

    eprintln!("\nCompiling windivert\n");
    if let Ok(out) = cmd.output() {
        if !out.status.success() {
            eprint!(
                "\nERROR: {:?}\n{}\n",
                &out.status,
                String::from_utf8_lossy(&out.stdout),
            );
            panic!()
        }
    } else {
        panic!("Error compiling windivert dll.");
    }

    let _ = fs::copy(
        format!(r#"{}\WinDivert.dll"#, &tmp_dir),
        format!(r#"{}\WinDivert.dll"#, &out_dir),
    );
    let _ = fs::copy(
        format!(r#"{}\WinDivert.lib"#, &tmp_dir),
        format!(r#"{}\WinDivert.lib"#, &out_dir),
    );
    if let Ok(dll_save_path) = env::var("WINDIVERT_DLL_OUTPUT") {
        let _ = fs::copy(
            format!(r#"{}\WinDivert.dll"#, &tmp_dir),
            format!(r#"{}\WinDivert.dll"#, &dll_save_path),
        );
        let _ = fs::copy(
            format!(r#"{}\WinDivert.lib"#, &tmp_dir),
            format!(r#"{}\WinDivert.lib"#, &dll_save_path),
        );
    }

    fs::remove_dir_all(&tmp_path).expect("Unable to delete temporary build folder");
}

fn gnu_compile(build: Build) {
    let compiler = build.get_compiler();

    let mut cmd = compiler.to_command();
    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search={}", &out_dir);

    let mangle = if env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "x86" {
        "_"
    } else {
        ""
    };
    set_gnu_c_options(&mut cmd);
    cmd.arg(format!("-Wl,--entry=${}WinDivertDllEntry", &mangle));
    cmd.args(&["-c", "vendor/dll/windivert.c"]);
    cmd.args(&["-o", &format!("{}/WinDivert.o", &out_dir)]);
    cmd.output().expect("Error compiling windivert c library");

    let mut cmd = build.get_compiler().to_command();
    set_gnu_c_options(&mut cmd);
    cmd.args(&["-o", &format!("{}/WinDivert.dll", &out_dir)]);
    cmd.args(&[
        &format!("{}/WinDivert.o", &out_dir),
        "vendor/dll/windivert.def",
        "-nostdlib",
    ]);
    set_gnu_c_libs(&mut cmd);
    cmd.output().expect("Error building windivert dll");

    let strip = Build::new()
        .get_compiler()
        .path()
        .to_string_lossy()
        .replace("gcc", "strip");
    let mut strip = Command::new(strip);
    strip.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    strip.arg(&format!("{}/WinDivert.dll", out_dir));
    let _ = strip.output().expect("Error striping windivert dll");

    let dlltool = Build::new()
        .get_compiler()
        .path()
        .to_string_lossy()
        .replace("gcc", "dlltool");
    let mut dlltool = Command::new(dlltool);
    dlltool.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    dlltool.args(&["--dllname", &format!("{}/WinDivert.dll", &out_dir)]);
    dlltool.args(&["--def", "vendor/dll/windivert.def"]);
    dlltool.args(&["--output-lib", &format!("{}/WinDivert.lib", &out_dir)]);
    let _ = dlltool.output().expect("Error building windivert lib");

    let _ = fs::remove_file(format!("{}/WinDivert.o", &out_dir));
}

fn set_gnu_c_options(cmd: &mut Command) {
    cmd.args(&[
        "-fno-ident",
        "-shared",
        "-Wall",
        "-Wno-pointer-to-int-cast",
        "-Os",
        "-Ivendor/include/",
        "-Wl,--enable-stdcall-fixup",
    ]);
}

fn set_gnu_c_libs(cmd: &mut Command) {
    cmd.args(&["-lkernel32", "-ladvapi32"]);
}
