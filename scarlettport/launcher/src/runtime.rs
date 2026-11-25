//! The runtime (xcl.exe, the D3D12 Agility SDK and DXC) is embedded in the launcher and
//! unpacked to %LOCALAPPDATA%\ScarlettPort\runtime on first run, or when its hash changes.
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const FILES: [(&str, &[u8]); 5] = [
    ("xcl.exe", include_bytes!("../runtime/xcl.exe")),
    ("D3D12/D3D12Core.dll", include_bytes!("../runtime/D3D12/D3D12Core.dll")),
    ("D3D12/d3d12SDKLayers.dll", include_bytes!("../runtime/D3D12/d3d12SDKLayers.dll")),
    ("dxcompiler.dll", include_bytes!("../runtime/dxcompiler.dll")),
    ("dxil.dll", include_bytes!("../runtime/dxil.dll")),
];

pub fn data_dir() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("ScarlettPort")
}

pub fn runtime_dir() -> PathBuf {
    data_dir().join("runtime")
}

pub fn xcl_exe() -> PathBuf {
    runtime_dir().join("xcl.exe")
}

/// Hash of the embedded runtime (computed once; it is what "shaders built for" refers to).
pub fn bundle_hash() -> &'static str {
    static H: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    H.get_or_init(compute_hash)
}

fn compute_hash() -> String {
    let mut h = Sha256::new();
    for (name, bytes) in FILES {
        h.update(name.as_bytes());
        h.update(Sha256::digest(bytes));
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

fn intact(dir: &Path) -> bool {
    FILES.iter().all(|(n, b)| std::fs::metadata(dir.join(n)).is_ok_and(|m| m.len() == b.len() as u64))
}

/// Extracts the runtime unless the on-disk copy already matches. Returns the runtime folder.
pub fn ensure() -> Result<PathBuf, String> {
    let dir = runtime_dir();
    let hash = bundle_hash();
    let marker = dir.join(".hash");
    if std::fs::read_to_string(&marker).is_ok_and(|v| v.trim() == hash) && intact(&dir) {
        return Ok(dir);
    }
    for (name, bytes) in FILES {
        let to = dir.join(name);
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        std::fs::write(&to, bytes).map_err(|e| format!("{}: {e}", to.display()))?;
    }
    std::fs::write(&marker, hash).map_err(|e| e.to_string())?;
    Ok(dir)
}
