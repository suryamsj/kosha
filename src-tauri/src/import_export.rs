use crate::config::{append_block, backup_config, colliding_alias, parse_config_text, render_host_block, HostEntry};
use serde::Serialize;
use std::fs;

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub accepted: Vec<HostEntry>,
    pub skipped: Vec<String>,
}

/// Determines which Host entries in `import_text` can be safely added on
/// top of `current_text`: an entry is accepted unless its alias collides
/// with an entry already in `current_text`, OR with an entry earlier in
/// `import_text` that was itself already accepted in this same pass (so
/// two colliding entries within one import file don't both get in).
fn plan_import(import_text: &str, current_text: &str) -> (Vec<HostEntry>, Vec<String>) {
    let mut known = parse_config_text(current_text);
    let candidates = parse_config_text(import_text);

    let mut accepted = vec![];
    let mut skipped = vec![];

    for candidate in candidates {
        if let Some(alias) = colliding_alias(&candidate.aliases, &known, None) {
            skipped.push(alias);
        } else {
            known.push(candidate.clone());
            accepted.push(candidate);
        }
    }

    (accepted, skipped)
}

#[tauri::command]
pub fn export_hosts() -> Result<String, String> {
    let ssh_dir = crate::keys::ssh_dir_path()?;
    let config_path = ssh_dir.join("config");
    let text = if config_path.exists() {
        fs::read_to_string(&config_path).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    let entries = parse_config_text(&text);
    let blocks: Vec<String> = entries
        .iter()
        .map(|entry| {
            render_host_block(
                &entry.aliases,
                entry.host_name.as_deref(),
                entry.user.as_deref(),
                entry.port.as_deref(),
                entry.identity_file.as_deref(),
            )
        })
        .collect();

    if blocks.is_empty() {
        Ok(String::new())
    } else {
        Ok(blocks.join("\n\n") + "\n")
    }
}

#[tauri::command]
pub fn preview_import(text: String) -> Result<ImportPreview, String> {
    let ssh_dir = crate::keys::ssh_dir_path()?;
    let config_path = ssh_dir.join("config");
    let current_text = if config_path.exists() {
        fs::read_to_string(&config_path).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    let (accepted, skipped) = plan_import(&text, &current_text);
    Ok(ImportPreview { accepted, skipped })
}

#[tauri::command]
pub fn import_hosts(text: String) -> Result<ImportPreview, String> {
    crate::keys::ensure_ssh_dir()?;
    let ssh_dir = crate::keys::ssh_dir_path()?;
    let config_path = ssh_dir.join("config");
    let current_text = if config_path.exists() {
        fs::read_to_string(&config_path).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    let (accepted, skipped) = plan_import(&text, &current_text);

    if !accepted.is_empty() {
        backup_config(&config_path)?;

        let mut new_text = current_text;
        for entry in &accepted {
            let block = render_host_block(
                &entry.aliases,
                entry.host_name.as_deref(),
                entry.user.as_deref(),
                entry.port.as_deref(),
                entry.identity_file.as_deref(),
            );
            new_text = append_block(&new_text, &block);
        }

        fs::write(&config_path, new_text).map_err(|e| e.to_string())?;
    }

    Ok(ImportPreview { accepted, skipped })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_import_accepts_all_new_hosts() {
        let import_text = "Host foo\n  HostName foo.com\n\nHost bar\n  HostName bar.com\n";
        let (accepted, skipped) = plan_import(import_text, "");
        assert_eq!(accepted.len(), 2);
        assert!(skipped.is_empty());
    }

    #[test]
    fn plan_import_skips_host_colliding_with_existing_config() {
        let current_text = "Host foo\n  HostName old.com\n";
        let import_text = "Host foo\n  HostName new.com\n";
        let (accepted, skipped) = plan_import(import_text, current_text);
        assert!(accepted.is_empty());
        assert_eq!(skipped, vec!["foo".to_string()]);
    }

    #[test]
    fn plan_import_accepts_and_skips_in_same_batch() {
        let current_text = "Host foo\n  HostName old.com\n";
        let import_text =
            "Host foo\n  HostName new.com\n\nHost bar\n  HostName bar.com\n";
        let (accepted, skipped) = plan_import(import_text, current_text);
        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted[0].aliases, vec!["bar".to_string()]);
        assert_eq!(skipped, vec!["foo".to_string()]);
    }

    #[test]
    fn plan_import_skips_second_of_two_colliding_entries_within_same_import() {
        let import_text =
            "Host dup\n  HostName first.com\n\nHost dup\n  HostName second.com\n";
        let (accepted, skipped) = plan_import(import_text, "");
        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted[0].host_name, Some("first.com".to_string()));
        assert_eq!(skipped, vec!["dup".to_string()]);
    }

    #[test]
    fn plan_import_reports_all_skipped_when_reimporting_own_export() {
        let current_text = "Host github\n  HostName github.com\n  User git\n  Port 22\n  IdentityFile ~/.ssh/id_ed25519\n\nHost work\n  HostName gitlab.company.com\n";
        let entries = parse_config_text(current_text);
        let blocks: Vec<String> = entries
            .iter()
            .map(|entry| {
                render_host_block(
                    &entry.aliases,
                    entry.host_name.as_deref(),
                    entry.user.as_deref(),
                    entry.port.as_deref(),
                    entry.identity_file.as_deref(),
                )
            })
            .collect();
        let exported_text = blocks.join("\n\n") + "\n";

        let (accepted, skipped) = plan_import(&exported_text, current_text);
        assert!(accepted.is_empty());
        assert_eq!(skipped.len(), 2);
    }
}
