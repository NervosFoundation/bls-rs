extern crate cc;
extern crate num_cpus;

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
    println!("cargo:rerun-if-changed=build.rs");
    if !Path::new("libpbc/.git").exists() {
        let _ = Command::new("git")
            .args(&["submodule", "update", "--init"])
            .status();
    }
    build_gmp();
    build_pbc();
}

fn build_gmp() {
    let src = env::current_dir()
        .expect("Can't find current dir")
        .join("libgmp");
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let build = dst.join("libgmp");
    t!(fs::create_dir_all(&build));
    cp_r(&src, &build);
    run_command_or_fail(&build, "./configure", &[]);
    run_command_or_fail(&build, "make", &["-j", &num_cpus::get().to_string()]);
}

fn build_pbc() {
    let current_dir = env::current_dir().expect("Can't find current dir");

    let pbc_src = current_dir.join("libpbc");
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let gmp = dst.join("libgmp");
    let pbc = dst.join("libpbc");

    let libs = dst.join("libs");
    let include_dir = dst.join("include");

    t!(fs::create_dir_all(&pbc));
    t!(fs::create_dir_all(&libs));
    t!(fs::create_dir_all(&include_dir));
    cp_r(&pbc_src, &pbc);
    cp_r(&pbc_src.join("include"), &include_dir);
    t!(fs::copy(gmp.join("gmp.h"), include_dir.join("gmp.h")));
    t!(fs::copy(gmp.join(".libs/libgmp.a"), libs.join("libgmp.a")));

    let ld = format!("-L{}", libs.display());
    let include = format!("-I{}", include_dir.display());

    let mut configure_flags = Vec::new();
    configure_flags.push("--enable-shared=no");
    configure_flags.push("--enable-static=yes");
    configure_flags.push("--with-pic");

    env::set_var("PBC_CPPFLAGS", &include);
    run_command_or_fail(&pbc, "./setup", &[]);

    Command::new("./configure")
        .current_dir(&pbc)
        .env("LDFLAGS", &ld)
        .env("CFLAGS", &include)
        .args(&configure_flags)
        .status()
        .expect("failed to execute process");
    Command::new("make")
        .current_dir(&pbc)
        .env("LDFLAGS", &ld)
        .env("CFLAGS", &include)
        .args(&["-j", &num_cpus::get().to_string()])
        .status()
        .expect("failed to execute process");
    t!(fs::copy(pbc.join(".libs/libpbc.a"), libs.join("libpbc.a")));

    println!("cargo:rustc-link-search=native={}", libs.display());
    println!("cargo:include={}", include_dir.display());
    println!("cargo:rustc-link-lib=static=pbc");
    println!("cargo:rustc-link-lib=static=gmp");

    cc::Build::new()
        .file("src/bls.c")
        .include(&include_dir)
        .static_flag(true)
        .compile("libbls.a");
    t!(fs::remove_dir_all(&gmp));
    t!(fs::remove_dir_all(&pbc));
}
