use std::process::Command;

/// Cleans up `ssh`'s stderr output for display: trims whitespace, and
/// falls back to a generic message if stderr came back empty.
fn clean_error_message(stderr: &str) -> String {
    let trimmed = stderr.trim();
    if trimmed.is_empty() {
        "Connection failed".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Builds the argument list for the connection-test `ssh` invocation, so
/// the required flags (in particular `BatchMode=yes`, which prevents an
/// interactive password prompt from hanging the app) are structurally
/// testable without spawning a real process.
fn ssh_args(alias: &str) -> Vec<String> {
    vec![
        "-o".to_string(),
        "BatchMode=yes".to_string(),
        "-o".to_string(),
        "ConnectTimeout=10".to_string(),
        "-o".to_string(),
        "ServerAliveInterval=5".to_string(),
        "-o".to_string(),
        "ServerAliveCountMax=2".to_string(),
        alias.to_string(),
        "exit".to_string(),
    ]
}

#[tauri::command]
pub fn test_connection(alias: String) -> Result<String, String> {
    let output = Command::new("ssh")
        .args(ssh_args(&alias))
        .output()
        .map_err(|_| "ssh not found — install OpenSSH".to_string())?;

    if output.status.success() {
        Ok("Connected successfully".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(clean_error_message(&stderr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_error_message_falls_back_on_empty_stderr() {
        assert_eq!(clean_error_message(""), "Connection failed");
    }

    #[test]
    fn clean_error_message_falls_back_on_whitespace_only_stderr() {
        assert_eq!(clean_error_message("   \n  "), "Connection failed");
    }

    #[test]
    fn clean_error_message_trims_real_message() {
        assert_eq!(
            clean_error_message("  Permission denied (publickey).  \n"),
            "Permission denied (publickey)."
        );
    }

    #[test]
    fn ssh_args_includes_required_safety_flags() {
        let args = ssh_args("myhost");
        assert!(args.windows(2).any(|w| w == ["-o", "BatchMode=yes"]));
        assert!(args.windows(2).any(|w| w == ["-o", "ConnectTimeout=10"]));
        assert!(args.windows(2).any(|w| w == ["-o", "ServerAliveInterval=5"]));
        assert!(args.windows(2).any(|w| w == ["-o", "ServerAliveCountMax=2"]));
        assert!(args.contains(&"myhost".to_string()));
        assert_eq!(args.last(), Some(&"exit".to_string()));
    }
}
