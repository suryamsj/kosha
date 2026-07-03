use std::fs;
use std::path::{Path, PathBuf};

#[derive(serde::Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HostEntry {
    pub aliases: Vec<String>,
    pub host_name: Option<String>,
    pub user: Option<String>,
    pub port: Option<String>,
    pub identity_file: Option<String>,
}

#[tauri::command]
pub fn list_hosts() -> Result<Vec<HostEntry>, String> {
    let ssh_dir = crate::keys::ssh_dir_path()?;
    let config_path = ssh_dir.join("config");
    if !config_path.exists() {
        return Ok(vec![]);
    }
    let text = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    Ok(parse_config_text(&text))
}

/// Parses `~/.ssh/config` syntax: a `Host <alias...>` line opens a block,
/// subsequent `Directive value` lines (whitespace-separated) populate it.
/// Unrecognized directives, blank lines, and comments are skipped.
pub(crate) fn parse_config_text(text: &str) -> Vec<HostEntry> {
    let mut entries = vec![];
    let mut current: Option<HostEntry> = None;

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.splitn(2, char::is_whitespace);
        let keyword = parts.next().unwrap_or("");
        let rest = parts.next().unwrap_or("").trim();

        if keyword.eq_ignore_ascii_case("host") {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            // Filter out wildcard aliases (containing * or ?)
            let aliases: Vec<String> = rest
                .split_whitespace()
                .filter(|s| !s.contains('*') && !s.contains('?'))
                .map(|s| s.to_string())
                .collect();

            if aliases.is_empty() {
                // No literal aliases remain; skip this block
                current = None;
            } else {
                // At least one literal alias remains
                current = Some(HostEntry {
                    aliases,
                    host_name: None,
                    user: None,
                    port: None,
                    identity_file: None,
                });
            }
            continue;
        }

        if let Some(entry) = current.as_mut() {
            match keyword.to_ascii_lowercase().as_str() {
                "hostname" => entry.host_name = Some(rest.to_string()),
                "user" => entry.user = Some(rest.to_string()),
                "port" => entry.port = Some(rest.to_string()),
                "identityfile" => entry.identity_file = Some(rest.to_string()),
                _ => {}
            }
        }
    }

    if let Some(entry) = current.take() {
        entries.push(entry);
    }

    entries
}

/// Resolves an `IdentityFile` value the way `ssh` does: `~/`-prefixed paths
/// expand against `home`, absolute paths pass through, anything else is
/// relative to `ssh_dir`.
pub(crate) fn resolve_identity_path(identity_file: &str, ssh_dir: &Path, home: &Path) -> PathBuf {
    if let Some(stripped) = identity_file.strip_prefix("~/") {
        home.join(stripped)
    } else if Path::new(identity_file).is_absolute() {
        PathBuf::from(identity_file)
    } else {
        ssh_dir.join(identity_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multi_alias_host_block() {
        let text = "Host foo bar\n  HostName example.com\n  User git\n";
        let entries = parse_config_text(text);
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].aliases,
            vec!["foo".to_string(), "bar".to_string()]
        );
        assert_eq!(entries[0].host_name, Some("example.com".to_string()));
        assert_eq!(entries[0].user, Some("git".to_string()));
    }

    #[test]
    fn parses_multiple_blocks_with_all_directives() {
        let text = "\
Host github
  HostName github.com
  User git
  Port 22
  IdentityFile ~/.ssh/id_ed25519

Host work
  HostName gitlab.company.com
  IdentityFile work_key
";
        let entries = parse_config_text(text);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].aliases, vec!["github".to_string()]);
        assert_eq!(entries[0].port, Some("22".to_string()));
        assert_eq!(
            entries[0].identity_file,
            Some("~/.ssh/id_ed25519".to_string())
        );
        assert_eq!(entries[1].aliases, vec!["work".to_string()]);
        assert_eq!(entries[1].identity_file, Some("work_key".to_string()));
    }

    #[test]
    fn skips_blank_lines_comments_and_unrecognized_directives() {
        let text = "\
# a comment
Host foo

  HostName example.com
  ForwardAgent yes
  Compression yes
";
        let entries = parse_config_text(text);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].host_name, Some("example.com".to_string()));
        assert_eq!(entries[0].user, None);
    }

    #[test]
    fn returns_empty_for_text_with_no_host_blocks() {
        assert_eq!(parse_config_text(""), Vec::new());
        assert_eq!(parse_config_text("# just a comment\n"), Vec::new());
    }

    #[test]
    fn resolves_tilde_prefixed_path() {
        let home = Path::new("/Users/oci");
        let ssh_dir = Path::new("/Users/oci/.ssh");
        assert_eq!(
            resolve_identity_path("~/.ssh/id_ed25519", ssh_dir, home),
            PathBuf::from("/Users/oci/.ssh/id_ed25519")
        );
    }

    #[test]
    fn resolves_absolute_path_passthrough() {
        let home = Path::new("/Users/oci");
        let ssh_dir = Path::new("/Users/oci/.ssh");
        assert_eq!(
            resolve_identity_path("/etc/ssh/special_key", ssh_dir, home),
            PathBuf::from("/etc/ssh/special_key")
        );
    }

    #[test]
    fn resolves_bare_filename_relative_to_ssh_dir() {
        let home = Path::new("/Users/oci");
        let ssh_dir = Path::new("/Users/oci/.ssh");
        assert_eq!(
            resolve_identity_path("work_key", ssh_dir, home),
            PathBuf::from("/Users/oci/.ssh/work_key")
        );
    }

    #[test]
    fn skips_pure_wildcard_host_block() {
        let text = "Host *\n  HostName example.com\n\nHost real\n  HostName real.example.com\n";
        let entries = parse_config_text(text);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].aliases, vec!["real".to_string()]);
    }

    #[test]
    fn keeps_literal_alias_and_drops_wildcard_alias_in_mixed_host_line() {
        let text = "Host foo *\n  HostName example.com\n";
        let entries = parse_config_text(text);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].aliases, vec!["foo".to_string()]);
    }
}
