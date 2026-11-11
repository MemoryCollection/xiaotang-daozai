//! 把岛仔图标作为 RT_GROUP_ICON #1 编进 exe，供 LoadImageW(hInst, 1, ...) 取用，
//! 同时生成与 Cargo.toml 同步的 VERSIONINFO（package.ps1 会校验 PE 版本）。
//! 不引入额外依赖：直接调用 Windows SDK 的 rc.exe，把 .res 交给链接器。

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn find_rc() -> Option<PathBuf> {
    let roots = [
        r"C:\Program Files (x86)\Windows Kits\10\bin",
        r"C:\Program Files\Windows Kits\10\bin",
    ];
    let mut found: Vec<PathBuf> = Vec::new();
    for root in roots {
        let Ok(entries) = fs::read_dir(root) else { continue };
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let rc = dir.join("x64").join("rc.exe");
            if rc.exists() {
                found.push(rc);
            }
        }
    }
    found.sort();
    found.pop()
}

/// "0.6.65" -> "0,6,65,0"；预发布版本号多余段截断，不足补 0。
fn version_numbers(version: &str) -> String {
    let mut parts: Vec<&str> = version
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .take(4)
        .collect();
    while parts.len() < 4 { parts.push("0"); }
    parts.join(",")
}

fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=assets/daozai.ico");
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out = out_dir.join("daozai.res");
    let Some(rc) = find_rc() else {
        println!("cargo:warning=rc.exe 未找到，跳过图标资源");
        return;
    };
    // app.rc 是模板：版本来自 Cargo.toml，避免升版本时忘记同步 PE 版本资源。
    // build script 的工作目录就是 crate 根目录，因此模板里的相对图标路径依然有效。
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let template = fs::read_to_string(manifest.join("app.rc")).expect("读取 app.rc 失败");
    let source = template
        .replace("@VERSION_NUM@", &version_numbers(&version))
        .replace("@VERSION@", &version);
    let generated = out_dir.join("app.generated.rc");
    fs::write(&generated, source).expect("写入生成的 app.rc 失败");
    let status = Command::new(&rc)
        .current_dir(&manifest)
        .arg("/nologo")
        .arg("/fo")
        .arg(&out)
        .arg(&generated)
        .status();
    if !status.map(|s| s.success()).unwrap_or(false) {
        panic!("rc.exe 编译 app.rc 失败");
    }
    println!("cargo:rustc-link-arg={}", out.display());
}
