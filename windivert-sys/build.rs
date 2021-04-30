use std::{
    env, fs,
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

fn generate_windows_bindings() {
    windows::build!(
        Windows::Win32::Debug::{
            WIN32_ERROR,
        },
        Windows::Devices::Custom::{
            IOControlAccessMode,
            IOControlBufferingMethod,
            IOControlCode,
        },
        Windows::Win32::FileSystem::{
            FILE_ACCESS_FLAGS,
            CancelIo,
        },
        Windows::Win32::Security::{
            SC_HANDLE,
            SERVICE_CHANGE_CONFIG,
            SERVICE_CONTROL_STOP,
            SERVICE_INTERROGATE,
            SERVICE_STATUS,
            SERVICE_QUERY_CONFIG,
            SERVICE_QUERY_STATUS,
            SERVICE_START,
            SERVICE_STOP,
            SERVICE_USER_DEFINED_CONTROL,
            CloseServiceHandle,
            ControlService,
            OpenServiceA,
            OpenSCManagerA,
        },
        Windows::Win32::SystemServices::{
            BOOL,
            FALSE,
            FILE_DEVICE_NETWORK,
            HANDLE,
            METHOD_OUT_DIRECT,
            OVERLAPPED,
            TRUE,
            WAIT_RETURN_CAUSE,
            CreateEventA,
            DeviceIoControl,
            GetOverlappedResultEx,
            TlsAlloc,
            TlsGetValue,
            TlsSetValue,
        },
        Windows::Win32::WindowsClustering::{
            CLCTL_CODES,
        },
    );
}

fn main() {
    generate_windows_bindings();

    if let Err(_) = std::env::var("DOCS_RS") {
        let out_dir = env::var("OUT_DIR").unwrap();
        println!("cargo:rerun-if-changed=wrapper.h");
        println!("cargo:rerun-if-env-changed=WINDIVERT_PATH");
        println!("cargo:rerun-if-env-changed=WINDIVERT_DLL_OUTPUT");
        println!("cargo:rerun-if-env-changed={}/WinDivert.dll", &out_dir);
        println!("cargo:rerun-if-env-changed={}/WinDivert.lib", &out_dir);

        println!("cargo:rustc-link-lib=dylib=WinDivert");
        println!("cargo:rustc-link-search=native={}", &out_dir);

        if let Ok(lib_path) = env::var("WINDIVERT_PATH") {
            println!("cargo:warning=Copying windivert dll, lib & sys files from the path provided if present.");
            for f in fs::read_dir(&lib_path).unwrap() {
                let file = f.unwrap();
                if let Some(name) = file.file_name().to_str() {
                    match name {
                        "WinDivert.dll" | "WinDivert.lib" | "WinDivert32.sys"
                        | "WinDivert64.sys" => {
                            let _ = fs::copy(file.path(), format!("{}/{}", &out_dir, &name));
                        }
                        _ => {}
                    }
                }
            }
        } else {
            println!("cargo:rustc-link-search=native={}", &out_dir);
            println!("cargo:warning=Environment variable WINDIVERT_PATH not found, building WinDivert from source.");
            build_windivert();
        };

        let arch = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_ref() {
            "x86" => "32",
            "x86_64" => "64",
            _ => panic!("Unsupported target architecture!"),
        };

        if let Err(_) = fs::metadata(format!("{}\\WinDivert{}.sys", &out_dir, &arch)) {
            println!(
                "cargo:warning=WinDivert{}.sys not found on the same directory as the dll.",
                arch
            )
        }
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

const MSVC_ARGS: &str = r#"/Ivendor\include /ZI /JMC /nologo /W1 /WX- /diagnostics:column /O1 /Oi /D WIN32 /D NDEBUG /D _WINDOWS /D _USRDLL /D DLL_EXPORTS /D _WINDLL /Gm- /EHsc /MDd /GS- /fp:precise /Zc:wchar_t /Zc:forScope /Zc:inline /Gd /TC /FC /errorReport:queue vendor\dll\windivert.c /link /ERRORREPORT:QUEUE /INCREMENTAL /NOLOGO kernel32.lib advapi32.lib /NODEFAULTLIB /DEF:vendor/dll/windivert.def /MANIFEST /manifest:embed /DEBUG:FASTLINK /TLDLIB:1 /ENTRY:"WinDivertDllEntry" /DYNAMICBASE /NXCOMPAT /DLL"#;
fn msvc_compile(build: Build) {
    let compiler = build.get_compiler();

    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search={}", &out_dir);

    let mut cmd = compiler.to_command();

    cmd.arg(format!(r#"/Fo{}\WinDivert.obj"#, &out_dir));
    cmd.arg(format!(r#"/Fd{}\WinDivert.pdb"#, &out_dir));

    cmd.args(MSVC_ARGS.split(" "));

    let arch = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_ref() {
        "x86" => "x86",
        "x86_64" => "x64",
        _ => panic!("Unsupported target architecture!"),
    };
    cmd.arg(format!("/MACHINE:{}", arch));

    cmd.arg(format!(r#"/PDB:{}\WinDivertDll.pdb"#, &out_dir));
    cmd.arg(format!(r#"/OUT:{}\WinDivert.dll"#, &out_dir));
    cmd.arg(format!(r#"/IMPLIB:{}\WinDivert.lib"#, &out_dir));

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

    if let Ok(dylib_save_dir) = env::var("WINDIVERT_DLL_OUTPUT") {
        let _ = fs::copy(
            format!(r#"{}\WinDivert.dll"#, &out_dir),
            format!(r#"{}\WinDivert.dll"#, &dylib_save_dir),
        );
        let _ = fs::copy(
            format!(r#"{}\WinDivert.lib"#, &out_dir),
            format!(r#"{}\WinDivert.lib"#, &dylib_save_dir),
        );
    } else {
        println!("cargo:warning=Environment variable WINDIVERT_DLL_OUTPUT not found, compiled dll & lib files will be stored on {}", &out_dir);
    };
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
