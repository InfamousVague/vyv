use std::process::Command;

const SUDOERS_FILE: &str = "/etc/sudoers.d/vyv_power";
const SUDOERS_CONTENT: &str = r#"Cmnd_Alias PMSET_VYV= /usr/bin/pmset -a disablesleep 1, /usr/bin/pmset -a disablesleep 0
%admin ALL=(ALL) NOPASSWD: PMSET_VYV
"#;

/// Check if the Vyv sudoers file is installed
pub fn is_sudoers_installed() -> bool {
    std::path::Path::new(SUDOERS_FILE).exists()
}

/// Install the sudoers file using osascript to prompt for admin password
pub fn install_sudoers() -> Result<(), String> {
    if is_sudoers_installed() {
        return Ok(());
    }

    // Use osascript to run a shell command with admin privileges
    // This shows the native macOS password dialog
    let script = format!(
        r#"do shell script "echo '{}' | sudo tee {} > /dev/null && sudo chmod 440 {}" with administrator privileges"#,
        SUDOERS_CONTENT.replace('\n', "\\n").replace('"', r#"\""#),
        SUDOERS_FILE,
        SUDOERS_FILE
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("Failed to run osascript: {}", e))?;

    if output.status.success() {
        eprintln!("[Vyv] Sudoers file installed successfully");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("User canceled") || stderr.contains("-128") {
            Err("User cancelled the authorization".to_string())
        } else {
            Err(format!("Failed to install sudoers: {}", stderr))
        }
    }
}

/// Enable clamshell sleep prevention (disable sleep entirely)
pub fn enable() -> Result<(), String> {
    if !is_sudoers_installed() {
        install_sudoers()?;
    }

    let output = Command::new("sudo")
        .args(["/usr/bin/pmset", "-a", "disablesleep", "1"])
        .output()
        .map_err(|e| format!("Failed to run pmset: {}", e))?;

    if output.status.success() {
        eprintln!("[Vyv] Clamshell sleep prevention enabled");
        Ok(())
    } else {
        Err(format!(
            "pmset failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// Disable clamshell sleep prevention (re-enable normal sleep)
pub fn disable() -> Result<(), String> {
    let output = Command::new("sudo")
        .args(["/usr/bin/pmset", "-a", "disablesleep", "0"])
        .output()
        .map_err(|e| format!("Failed to run pmset: {}", e))?;

    if output.status.success() {
        eprintln!("[Vyv] Clamshell sleep prevention disabled");
        Ok(())
    } else {
        Err(format!(
            "pmset failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// Check if disablesleep is currently active
pub fn is_enabled() -> bool {
    let output = Command::new("pmset")
        .args(["-g"])
        .output()
        .ok();

    if let Some(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Look for "SleepDisabled  1"
        stdout.lines().any(|line| {
            line.contains("SleepDisabled") && line.trim().ends_with('1')
        })
    } else {
        false
    }
}
