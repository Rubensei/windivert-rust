use std::{
    env,
    process::{Command, Stdio},
};

use cc::{Build, Tool};

fn print_env(compiler: &Tool) {
    eprintln!("Environment variables:");
    for (k, v) in env::vars() {
        eprintln!("{}={}", k, v);
    }
    eprintln!("\nCompiler:\n{}", compiler.path().to_string_lossy());
    eprintln!("\nCompiler arguments:");
    for arg in compiler.args().iter() {
        eprintln!("{}", arg.to_string_lossy());
    }
    eprintln!("\nCompiler environment variables:");
    for (k, v) in compiler.env().iter() {
        eprintln!("{}={}", k.to_string_lossy(), v.to_string_lossy());
    }
    eprintln!("");
}

fn main() {
    if let Err(_) = std::env::var("DOCS_RS") {
        println!("cargo:rerun-if-changed=wrapper.h");

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
        todo!("Missing msvc build script");
    } else if compiler.is_like_gnu() {
        gnu_compile(build);
    }
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
    strip.arg(&format!("{}/WinDivert.dll", out_dir));
    let _ = strip.output().expect("Error striping windivert dll");

    let dlltool = Build::new()
        .get_compiler()
        .path()
        .to_string_lossy()
        .replace("gcc", "dlltool");
    let mut dlltool = Command::new(dlltool);
    dlltool.args(&["--dllname", &format!("{}/WinDivert.dll", &out_dir)]);
    dlltool.args(&["--def", "vendor/dll/windivert.def"]);
    dlltool.args(&["--output-lib", &format!("{}/WinDivert.lib", &out_dir)]);
    let _ = dlltool.output().expect("Error building windivert lib");

    let _ = std::fs::remove_file(format!("{}/WinDivert.o", &out_dir));
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
