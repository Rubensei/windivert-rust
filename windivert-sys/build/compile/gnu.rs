use cc::Build;
use path_macro2::{path, path_const};
use std::process::{Command, Stdio};
use std::{env, fs};

pub fn lib() {
    let mut build = Build::new();
    let out_dir = env::var("OUT_DIR").unwrap();

    build
        .out_dir(&out_dir)
        .include(path_const!(vendor / include))
        .file(path_const!(vendor / dll / windivert.c));

    build.compile("WinDivert");
}

pub fn dll() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let compiler = Build::new().get_compiler();

    let mut cmd = compiler.to_command();
    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    let mangle = if env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "x86" {
        "_"
    } else {
        ""
    };
    cmd.args(DYNAMIC_C_OPTIONS);
    cmd.arg(format!("-Wl,--entry=${mangle}WinDivertDllEntry"));
    cmd.args(["-c", path_const!(vendor / dll / windivert.c)]);
    cmd.args(["-o", &path!({ out_dir } / WinDivert.o).to_string_lossy()]);
    cmd.output().expect("Error compiling windivert c library");

    let mut cmd = Build::new().get_compiler().to_command();
    cmd.args(DYNAMIC_C_OPTIONS);
    cmd.args(["-o", &path!({ out_dir } / WinDivert.dll).to_string_lossy()]);
    cmd.args([
        &path!({ out_dir } / WinDivert.o).to_string_lossy(),
        path_const!(vendor / dll / windivert.def),
        "-nostdlib",
    ]);
    cmd.args(DYNAMIC_C_INCLUDES);
    cmd.output().expect("Error building windivert dll");

    let strip = Build::new()
        .get_compiler()
        .path()
        .to_string_lossy()
        .replace("gcc", "strip");

    let mut strip = Command::new(strip);
    strip.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    strip.arg(path!({ out_dir } / WinDivert.dll));
    let _ = strip.output().expect("Error striping windivert dll");

    let dlltool = Build::new()
        .get_compiler()
        .path()
        .to_string_lossy()
        .replace("gcc", "dlltool");
    let mut dlltool = Command::new(dlltool);
    dlltool.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    dlltool.args([
        "--dllname",
        &path!({ out_dir } / WinDivert.dll).to_string_lossy(),
    ]);
    dlltool.args(["--def", path_const!(vendor / dll / windivert.def)]);
    dlltool.args([
        "--output-lib",
        &path!({ out_dir } / WinDivert.lib).to_string_lossy(),
    ]);
    let _ = dlltool.output().expect("Error building windivert lib");

    // Clean up object file
    let _ = fs::remove_file(path!({ out_dir } / WinDivert.o));
}

const DYNAMIC_C_OPTIONS: &[&str] = &[
    r#"-fno-ident"#,
    r#"-shared"#,
    r#"-Wall"#,
    r#"-Wno-pointer-to-int-cast"#,
    r#"-Os"#,
    concat!("-I", path_const!(vendor / include /)),
    r#"-Wl,--enable-stdcall-fixup"#,
];

const DYNAMIC_C_INCLUDES: &[&str] = &[r#"-lkernel32"#, r#"-ladvapi32"#];
