use std::{
    env, fs,
    process::{Command, Stdio},
};

use cc::Build;

pub fn compile(build: Build) {
    let compiler = build.get_compiler();

    let mut cmd = compiler.to_command();
    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search={out_dir}");

    let mangle = if env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "x86" {
        "_"
    } else {
        ""
    };
    set_gnu_c_options(&mut cmd);
    cmd.arg(format!("-Wl,--entry=${mangle}WinDivertDllEntry"));
    cmd.args(&["-c", "vendor/dll/windivert.c"]);
    cmd.args(&["-o", &format!("{out_dir}/WinDivert.o")]);
    cmd.output().expect("Error compiling windivert c library");

    let mut cmd = build.get_compiler().to_command();
    set_gnu_c_options(&mut cmd);
    cmd.args(&["-o", &format!("{out_dir}/WinDivert.dll")]);
    cmd.args(&[
        &format!("{out_dir}/WinDivert.o"),
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

    strip.arg(&format!("{out_dir}/WinDivert.dll"));
    let _ = strip.output().expect("Error striping windivert dll");

    let dlltool = Build::new()
        .get_compiler()
        .path()
        .to_string_lossy()
        .replace("gcc", "dlltool");
    let mut dlltool = Command::new(dlltool);
    dlltool.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    dlltool.args(&["--dllname", &format!("{out_dir}/WinDivert.dll")]);
    dlltool.args(&["--def", "vendor/dll/windivert.def"]);
    dlltool.args(&["--output-lib", &format!("{out_dir}/WinDivert.lib")]);
    let _ = dlltool.output().expect("Error building windivert lib");

    let _ = fs::remove_file(format!("{out_dir}/WinDivert.o"));
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
