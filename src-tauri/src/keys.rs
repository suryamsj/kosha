use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct KeyInfo {
    pub name: String,
    pub key_type: String,
    pub fingerprint: String,
    pub created_at: u64,
    pub has_private: bool,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListKeysResponse {
    pub ssh_dir_missing: bool,
    pub keys: Vec<KeyInfo>,
}

pub(crate) fn ssh_dir_path() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|home| home.join(".ssh"))
        .ok_or_else(|| "Could not determine home directory".to_string())
}

/// Extracts and sorts key names from a list of filenames in `~/.ssh`,
/// keeping only those with a `.pub` counterpart.
pub(crate) fn pub_key_names(filenames: &[String]) -> Vec<String> {
    let mut names: Vec<String> = filenames
        .iter()
        .filter_map(|f| f.strip_suffix(".pub").map(|s| s.to_string()))
        .collect();
    names.sort();
    names
}

/// Parses one line of `ssh-keygen -lf <pubkey>` output, e.g.
/// "256 SHA256:abcDEF123 user@host (ED25519)" -> ("ED25519", "SHA256:abcDEF123")
pub(crate) fn parse_fingerprint_line(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    let fingerprint = parts[1].to_string();
    let key_type = line
        .rsplit('(')
        .next()
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or("unknown")
        .to_string();
    Some((key_type, fingerprint))
}

fn fingerprint_for(pub_path: &Path) -> Option<(String, String)> {
    let output = Command::new("ssh-keygen")
        .arg("-lf")
        .arg(pub_path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_fingerprint_line(stdout.trim())
}

fn created_at_secs(path: &Path) -> u64 {
    fs::metadata(path)
        .and_then(|m| m.created().or_else(|_| m.modified()))
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[tauri::command]
pub fn list_keys() -> Result<ListKeysResponse, String> {
    let ssh_dir = ssh_dir_path()?;
    if !ssh_dir.exists() {
        return Ok(ListKeysResponse {
            ssh_dir_missing: true,
            keys: vec![],
        });
    }

    let entries = fs::read_dir(&ssh_dir).map_err(|e| e.to_string())?;
    let filenames: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    let keys = pub_key_names(&filenames)
        .into_iter()
        .map(|name| {
            let pub_path = ssh_dir.join(format!("{name}.pub"));
            let priv_path = ssh_dir.join(&name);
            let (key_type, fingerprint) = fingerprint_for(&pub_path)
                .unwrap_or_else(|| ("unknown".to_string(), "unknown".to_string()));
            KeyInfo {
                name,
                key_type,
                fingerprint,
                created_at: created_at_secs(&pub_path),
                has_private: priv_path.exists(),
            }
        })
        .collect();

    Ok(ListKeysResponse {
        ssh_dir_missing: false,
        keys,
    })
}

#[tauri::command]
pub fn ensure_ssh_dir() -> Result<(), String> {
    let ssh_dir = ssh_dir_path()?;
    if !ssh_dir.exists() {
        fs::create_dir_all(&ssh_dir).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&ssh_dir, fs::Permissions::from_mode(0o700))
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn generate_key(name: String, passphrase: Option<String>) -> Result<KeyInfo, String> {
    ensure_ssh_dir()?;
    let ssh_dir = ssh_dir_path()?;
    let priv_path = ssh_dir.join(&name);
    let pub_path = ssh_dir.join(format!("{name}.pub"));

    if priv_path.exists() || pub_path.exists() {
        return Err(format!("Key '{name}' already exists"));
    }

    let status = Command::new("ssh-keygen")
        .arg("-t")
        .arg("ed25519")
        .arg("-f")
        .arg(&priv_path)
        .arg("-N")
        .arg(passphrase.unwrap_or_default())
        .arg("-C")
        .arg(&name)
        .status()
        .map_err(|_| "ssh-keygen not found — install OpenSSH".to_string())?;

    if !status.success() {
        return Err("ssh-keygen failed to generate key".to_string());
    }

    let (key_type, fingerprint) = fingerprint_for(&pub_path)
        .unwrap_or_else(|| ("unknown".to_string(), "unknown".to_string()));

    Ok(KeyInfo {
        name,
        key_type,
        fingerprint,
        created_at: created_at_secs(&pub_path),
        has_private: true,
    })
}

#[tauri::command]
pub fn get_public_key(name: String) -> Result<String, String> {
    let ssh_dir = ssh_dir_path()?;
    let pub_path = ssh_dir.join(format!("{name}.pub"));
    fs::read_to_string(&pub_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_key(name: String) -> Result<(), String> {
    let ssh_dir = ssh_dir_path()?;
    let priv_path = ssh_dir.join(&name);
    let pub_path = ssh_dir.join(format!("{name}.pub"));

    if priv_path.exists() {
        fs::remove_file(&priv_path).map_err(|e| e.to_string())?;
    }
    if pub_path.exists() {
        fs::remove_file(&pub_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ed25519_fingerprint_line() {
        let line = "256 SHA256:abcDEF123 user@host (ED25519)";
        assert_eq!(
            parse_fingerprint_line(line),
            Some(("ED25519".to_string(), "SHA256:abcDEF123".to_string()))
        );
    }

    #[test]
    fn returns_none_for_malformed_line() {
        assert_eq!(parse_fingerprint_line(""), None);
        assert_eq!(parse_fingerprint_line("garbage"), None);
    }

    #[test]
    fn extracts_and_sorts_pub_key_names() {
        let files = vec![
            "id_rsa".to_string(),
            "work.pub".to_string(),
            "id_rsa.pub".to_string(),
            "config".to_string(),
        ];
        assert_eq!(
            pub_key_names(&files),
            vec!["id_rsa".to_string(), "work".to_string()]
        );
    }

    #[test]
    fn ignores_files_without_pub_suffix() {
        let files = vec!["known_hosts".to_string(), "config".to_string()];
        assert_eq!(pub_key_names(&files), Vec::<String>::new());
    }
}
