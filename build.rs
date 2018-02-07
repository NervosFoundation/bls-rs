extern crate cc;

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(n) => n,
        Err(e) => panic!("\n{} failed with {}\n", stringify!($e), e),
    })
}

fn cp_r(dir: &Path, dst: &Path) {
    for entry in t!(fs::read_dir(dir)) {
        let entry = t!(entry);
        let path = entry.path();
        let dst = dst.join(path.file_name().unwrap());
        if t!(fs::metadata(&path)).is_file() {
            t!(fs::copy(path, dst));
        } else {
            t!(fs::create_dir_all(&dst));
            cp_r(&path, &dst);
        }
    }
}

fn run_command_or_fail<P: AsRef<Path>>(dir: P, cmd: &str, args: &[&str]) {
    println_stderr!(
        "Running command: \"{} {}\" in dir: {}",
        cmd,
        args.join(" "),
        dir.as_ref().display()
    );
    let ret = Command::new(cmd).current_dir(dir).args(args).status();
    match ret.map(|status| (status.success(), status.code())) {
        Ok((true, _)) => return,
        Ok((false, Some(c))) => panic!("Command failed with error code {}", c),
        Ok((false, None)) => panic!("Command got killed"),
        Err(e) => panic!("Command failed with error: {}", e),
    }
}

fn main() {
    let target = std::env::var("TARGET").unwrap();
    if target.contains("linux") {
        println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
    }
    println!("cargo:rustc-link-lib=static=gmp");
    if !Path::new("libpbc/.git").exists() {
        let _ = Command::new("git").args(&["submodule", "update", "--init"])
                                   .status();
    }

    build_pbc();
}

fn build_pbc() {
    let src = env::current_dir()
        .expect("Can't find current dir")
        .join("libpbc");
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let build = dst.join("build");

    t!(fs::create_dir_all(&build));
    cp_r(&src, &build);

    let mut configure_flags = Vec::new();
    configure_flags.push("--enable-shared=no");
    configure_flags.push("--enable-static=yes");

    run_command_or_fail(&build, "./setup", &[]);
    println!("Configuring libpbc");
    run_command_or_fail(&build, "./configure", &configure_flags);
    println!("Compiling libpbc");
    run_command_or_fail(&build, "make", &[]);

    println!("cargo:rustc-link-search=native={}/.libs", build.display());
    println!("cargo:rustc-link-lib=static=pbc");

    let include_dir = format!("{}/include", build.display());
    cc::Build::new()
        .file("src/bls.c")
        .flag("-fPIC")
        .include(&include_dir)
        .static_flag(true)
        .compile("libbls.a");
}
