use std::{env, fs, path::PathBuf, process::Command};

fn has_loongarch_intrinsics() -> bool {
    let out_dir = match env::var_os("OUT_DIR") {
        Some(path) => PathBuf::from(path),
        None => return false,
    };
    let manifest_dir = match env::var_os("CARGO_MANIFEST_DIR") {
        Some(path) => PathBuf::from(path),
        None => return false,
    };
    let rustc = match env::var_os("RUSTC") {
        Some(path) => path,
        None => return false,
    };
    let target = match env::var_os("TARGET") {
        Some(target) => target,
        None => return false,
    };

    // Check the actual target APIs without linking or enabling unstable features.
    let mut source = format!(
        "#![no_std]\n#[path = {:?}]\nmod loongarch64;\n",
        manifest_dir.join("src/binary16/arch/loongarch64.rs")
    );
    if env::var_os("CARGO_FEATURE_STD").is_some() {
        source.push_str(
            "extern crate std;\n\
             pub fn detect_lsx() -> bool { std::arch::is_loongarch_feature_detected!(\"lsx\") }\n",
        );
    }
    let probe = out_dir.join("loongarch_intrinsics.rs");
    if fs::write(&probe, source).is_err() {
        return false;
    }

    let mut command = Command::new(rustc);
    if let Ok(flags) = env::var("CARGO_ENCODED_RUSTFLAGS") {
        command.args(flags.split('\x1f').filter(|flag| !flag.is_empty()));
    }
    command
        .args(["--crate-name", "float16_loongarch_intrinsics", "--crate-type", "lib"])
        .args(["--edition", "2021", "--emit", "metadata", "--target"])
        .arg(target)
        .arg("--out-dir")
        .arg(out_dir)
        .arg(probe)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn main() {
    let version = rustc_version::version().unwrap();

    if version >= rustc_version::Version::parse("1.63.0").unwrap() {
        println!("cargo:rustc-cfg=has_aarch64_intrinsics");
    }

    if version >= rustc_version::Version::parse("1.70.0").unwrap() {
        println!("cargo:rustc-cfg=has_x86_intrinsics");
    }

    if env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("loongarch64")
        && has_loongarch_intrinsics()
    {
        println!("cargo:rustc-cfg=has_loongarch_intrinsics");
    }
}
