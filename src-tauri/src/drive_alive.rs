use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio_util::sync::CancellationToken;

const KEEPALIVE_FILE: &str = ".vyv_keepalive";
const PING_INTERVAL_SECS: u64 = 60;

/// List mounted external volumes (macOS: everything in /Volumes except the boot volume).
pub fn list_drives() -> Vec<PathBuf> {
    let volumes = Path::new("/Volumes");
    let mut drives = Vec::new();

    if let Ok(entries) = std::fs::read_dir(volumes) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Skip the boot volume (symlink target of /)
            if is_boot_volume(&path) {
                continue;
            }
            if path.is_dir() {
                drives.push(path);
            }
        }
    }

    drives.sort();
    drives
}

/// Spawn a background task that pings each selected drive every ~60s.
pub fn start(drives: Vec<PathBuf>, cancel: CancellationToken) {
    tauri::async_runtime::spawn(async move {
        loop {
            // Ping all drives
            for drive in &drives {
                let _ = ping_drive(drive);
            }

            // Wait for the next interval or cancellation
            tokio::select! {
                _ = cancel.cancelled() => {
                    // Clean up keepalive files on cancellation
                    for drive in &drives {
                        let _ = cleanup_drive(drive);
                    }
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(PING_INTERVAL_SECS)) => {}
            }
        }
    });
}

/// Write a tiny file and immediately remove it to generate disk activity.
fn ping_drive(drive: &Path) -> std::io::Result<()> {
    let file_path = drive.join(KEEPALIVE_FILE);
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    std::fs::write(&file_path, timestamp.to_string().as_bytes())?;
    std::fs::remove_file(&file_path)?;
    Ok(())
}

/// Remove any leftover keepalive file.
fn cleanup_drive(drive: &Path) {
    let file_path = drive.join(KEEPALIVE_FILE);
    let _ = std::fs::remove_file(file_path);
}

/// Check whether a volume path is the macOS boot volume.
fn is_boot_volume(path: &Path) -> bool {
    // On macOS, "/" is the boot volume. /Volumes contains a symlink to it.
    match std::fs::read_link(path) {
        Ok(target) => target == Path::new("/"),
        Err(_) => false,
    }
}
