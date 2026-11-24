//! Stages the prebuilt runtime into launcher/runtime (embedded by src/runtime.rs) and stamps
//! the Windows resources (icon, version info).
//! Sources can be overridden with XCL_SRC (folder with xcl.exe and D3D12\) and DXC_SRC
//! (folder with dxcompiler.dll and dxil.dll).
//! Set SCARLETT_NO_EMBED=1 to embed empty placeholders instead (CI builds; the result cannot run a game).
use std::path::{Path, PathBuf};

fn stage(src: &Path, name: &str, dst: &Path) {
    let to = dst.join(name);
    if to.is_file() {
        return;
    }
    let from = src.join(name);
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::copy(&from, &to).unwrap_or_else(|e| {
        panic!("runtime file {} is missing ({e}). Put it in launcher/runtime/ or set XCL_SRC / DXC_SRC.", from.display())
    });
}

fn stage_empty(name: &str, dst: &Path) {
    let to = dst.join(name);
    if to.is_file() {
        return;
    }
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&to, b"").unwrap();
}

fn main() {
    let dst = PathBuf::from("runtime");
    println!("cargo:rerun-if-env-changed=SCARLETT_NO_EMBED");
    if std::env::var_os("SCARLETT_NO_EMBED").is_some() {
        for name in ["xcl.exe", "D3D12/D3D12Core.dll", "D3D12/d3d12SDKLayers.dll", "dxcompiler.dll", "dxil.dll"] {
            stage_empty(name, &dst);
        }
    } else {
        stage_all(&dst);
    }
    println!("cargo:rerun-if-changed=runtime");
    println!("cargo:rerun-if-changed=assets/app.ico");
    println!("cargo:rerun-if-env-changed=XCL_SRC");
    println!("cargo:rerun-if-env-changed=DXC_SRC");
    stamp_resources();
}

fn stage_all(dst: &Path) {
    let xcl = PathBuf::from(std::env::var("XCL_SRC").unwrap_or_else(|_| "M:/GTA6/xcl-test".into()));
    let dxc = PathBuf::from(std::env::var("DXC_SRC").unwrap_or_else(|_| {
        "C:/Program Files (x86)/Windows Kits/10/bin/10.0.26100.0/x64".into()
    }));
    stage(&xcl, "xcl.exe", dst);
    stage(&xcl, "D3D12/D3D12Core.dll", dst);
    stage(&xcl, "D3D12/d3d12SDKLayers.dll", dst);
    stage(&dxc, "dxcompiler.dll", dst);
    stage(&dxc, "dxil.dll", dst);
}

fn stamp_resources() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/app.ico")
            .set("ProductName", "ScarlettPort")
            .set("FileDescription", "ScarlettPort launcher")
            .set("CompanyName", "ScarlettPort")
            .set("InternalName", "ScarlettPort")
            .set("OriginalFilename", "ScarlettPort.exe")
            .set("LegalCopyright", "ScarlettPort project (MIT)");
        if let Err(e) = res.compile() {
            println!("cargo:warning=Windows resources not embedded: {e}");
        }
    }
}
