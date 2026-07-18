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
    parse_config_with_ranges(text)
        .into_iter()
        .map(|(entry, _)| entry)
        .collect()
}

/// Same parsing as `parse_config_text`, but also returns each entry's
/// `[start_line, end_line)` line-index range in `text` (0-indexed, end
/// exclusive — a block's range includes any blank/comment lines up to but
/// not including the next `Host` line, or end of file). Used by the write
/// path to locate exactly which lines to replace or remove without
/// touching anything else in the file.
pub(crate) fn parse_config_with_ranges(text: &str) -> Vec<(HostEntry, (usize, usize))> {
    let mut entries: Vec<(HostEntry, (usize, usize))> = vec![];
    let mut current: Option<(HostEntry, usize)> = None;

    let lines: Vec<&str> = text.lines().collect();

    for (i, raw_line) in lines.iter().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.splitn(2, char::is_whitespace);
        let keyword = parts.next().unwrap_or("");
        let rest = parts.next().unwrap_or("").trim();

        if keyword.eq_ignore_ascii_case("host") {
            if let Some((entry, start)) = current.take() {
                entries.push((entry, (start, i)));
            }
            let aliases: Vec<String> = rest
                .split_whitespace()
                .filter(|s| !s.contains('*') && !s.contains('?'))
                .map(|s| s.to_string())
                .collect();

            current = if aliases.is_empty() {
                None
            } else {
                Some((
                    HostEntry {
                        aliases,
                        host_name: None,
                        user: None,
                        port: None,
                        identity_file: None,
                    },
                    i,
                ))
            };
            continue;
        }

        if let Some((entry, _)) = current.as_mut() {
            match keyword.to_ascii_lowercase().as_str() {
                "hostname" => entry.host_name = Some(rest.to_string()),
                "user" => entry.user = Some(rest.to_string()),
                "port" => entry.port = Some(rest.to_string()),
                "identityfile" => entry.identity_file = Some(rest.to_string()),
                _ => {}
            }
        }
    }

    if let Some((entry, start)) = current.take() {
        entries.push((entry, (start, lines.len())));
    }

    entries
}

/// Finds the line range of the Host block whose aliases exactly equal
/// `aliases`, by re-parsing `text` fresh. Returns `None` if no block
/// matches (e.g. the file changed since the caller last read it).
pub(crate) fn find_host_range(text: &str, aliases: &[String]) -> Option<(usize, usize)> {
    parse_config_with_ranges(text)
        .into_iter()
        .find(|(entry, _)| entry.aliases == aliases)
        .map(|(_, range)| range)
}

/// Renders a `Host` block in the format the parser expects: a `Host` line
/// listing all aliases, then one indented `Directive value` line per
/// `Some` field, in HostName/User/Port/IdentityFile order. `None` fields
/// are omitted entirely (no empty directive lines).
pub(crate) fn render_host_block(
    aliases: &[String],
    host_name: Option<&str>,
    user: Option<&str>,
    port: Option<&str>,
    identity_file: Option<&str>,
) -> String {
    let mut lines = vec![format!("Host {}", aliases.join(" "))];
    if let Some(v) = host_name {
        lines.push(format!("  HostName {v}"));
    }
    if let Some(v) = user {
        lines.push(format!("  User {v}"));
    }
    if let Some(v) = port {
        lines.push(format!("  Port {v}"));
    }
    if let Some(v) = identity_file {
        lines.push(format!("  IdentityFile {v}"));
    }
    lines.join("\n")
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

/// Appends `block` to `existing_text`, adding blank-line separation so the
/// new block doesn't run into the previous content. Handles an empty
/// starting file (no separator needed) and a file that doesn't yet end in
/// a newline.
pub(crate) fn append_block(existing_text: &str, block: &str) -> String {
    let mut new_text = existing_text.to_string();
    if !new_text.is_empty() && !new_text.ends_with('\n') {
        new_text.push('\n');
    }
    if !new_text.is_empty() {
        new_text.push('\n');
    }
    new_text.push_str(block);
    new_text.push('\n');
    new_text
}

/// Replaces lines `[start, end)` of `text` with `block`'s lines, leaving
/// every other line untouched.
fn replace_lines(text: &str, range: (usize, usize), block: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let (start, end) = range;
    let mut result: Vec<&str> = Vec::new();
    result.extend(&lines[..start]);
    result.extend(block.lines());
    result.extend(&lines[end..]);
    result.join("\n") + "\n"
}

/// Removes lines `[start, end)` from `text`, leaving every other line
/// untouched.
fn remove_lines(text: &str, range: (usize, usize)) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let (start, end) = range;
    let mut result: Vec<&str> = Vec::new();
    result.extend(&lines[..start]);
    result.extend(&lines[end..]);
    if result.is_empty() {
        String::new()
    } else {
        result.join("\n") + "\n"
    }
}

/// Returns the first alias in `aliases` that's already used by a
/// *different* existing host block. `exclude`, if given, is the alias
/// list of the block being edited — that block is not itself considered
/// a collision.
pub(crate) fn colliding_alias(
    aliases: &[String],
    existing: &[HostEntry],
    exclude: Option<&[String]>,
) -> Option<String> {
    for entry in existing {
        if let Some(exclude) = exclude {
            if entry.aliases == exclude {
                continue;
            }
        }
        for alias in aliases {
            if entry.aliases.contains(alias) {
                return Some(alias.clone());
            }
        }
    }
    None
}

/// True if any alias contains a wildcard character (`*` or `?`).
pub(crate) fn has_wildcard_alias(aliases: &[String]) -> bool {
    aliases.iter().any(|a| a.contains('*') || a.contains('?'))
}

/// Backs up `config_path` to `config.bak.<unix_seconds>` alongside it, if
/// `config_path` currently exists. No-op (`Ok`) if there's nothing to back
/// up yet (a config being created for the first time).
pub(crate) fn backup_config(config_path: &Path) -> Result<(), String> {
    if !config_path.exists() {
        return Ok(());
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let backup_path = config_path.with_file_name(format!("config.bak.{nanos}"));
    fs::copy(config_path, &backup_path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn add_host(
    aliases: Vec<String>,
    host_name: Option<String>,
    user: Option<String>,
    port: Option<String>,
    identity_file: Option<String>,
) -> Result<(), String> {
    if has_wildcard_alias(&aliases) {
        return Err("Host alias cannot contain wildcards".to_string());
    }

    crate::keys::ensure_ssh_dir()?;
    let ssh_dir = crate::keys::ssh_dir_path()?;
    let config_path = ssh_dir.join("config");

    let text = if config_path.exists() {
        fs::read_to_string(&config_path).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    let existing = parse_config_text(&text);
    if let Some(alias) = colliding_alias(&aliases, &existing, None) {
        return Err(format!("Alias '{alias}' already used by another host"));
    }

    backup_config(&config_path)?;

    let block = render_host_block(
        &aliases,
        host_name.as_deref(),
        user.as_deref(),
        port.as_deref(),
        identity_file.as_deref(),
    );
    let new_text = append_block(&text, &block);

    fs::write(&config_path, new_text).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn edit_host(
    original_aliases: Vec<String>,
    aliases: Vec<String>,
    host_name: Option<String>,
    user: Option<String>,
    port: Option<String>,
    identity_file: Option<String>,
) -> Result<(), String> {
    if has_wildcard_alias(&aliases) {
        return Err("Host alias cannot contain wildcards".to_string());
    }

    let ssh_dir = crate::keys::ssh_dir_path()?;
    let config_path = ssh_dir.join("config");
    if !config_path.exists() {
        return Err("Host block not found — config may have changed".to_string());
    }
    let text = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;

    let existing = parse_config_text(&text);
    if let Some(alias) = colliding_alias(&aliases, &existing, Some(&original_aliases)) {
        return Err(format!("Alias '{alias}' already used by another host"));
    }

    let range = find_host_range(&text, &original_aliases)
        .ok_or_else(|| "Host block not found — config may have changed".to_string())?;

    backup_config(&config_path)?;

    let block = render_host_block(
        &aliases,
        host_name.as_deref(),
        user.as_deref(),
        port.as_deref(),
        identity_file.as_deref(),
    );
    let new_text = replace_lines(&text, range, &block);

    fs::write(&config_path, new_text).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_host(aliases: Vec<String>) -> Result<(), String> {
    let ssh_dir = crate::keys::ssh_dir_path()?;
    let config_path = ssh_dir.join("config");
    if !config_path.exists() {
        return Err("Host block not found — config may have changed".to_string());
    }
    let text = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;

    let range = find_host_range(&text, &aliases)
        .ok_or_else(|| "Host block not found — config may have changed".to_string())?;

    backup_config(&config_path)?;

    let new_text = remove_lines(&text, range);
    fs::write(&config_path, new_text).map_err(|e| e.to_string())
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

    #[test]
    fn parse_config_with_ranges_reports_correct_line_ranges() {
        let text = "Host foo\n  HostName example.com\n\nHost bar\n  HostName other.com\n";
        let entries = parse_config_with_ranges(text);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].1, (0, 3));
        assert_eq!(entries[1].1, (3, 5));
    }

    #[test]
    fn find_host_range_locates_correct_block_among_several() {
        let text = "Host foo\n  HostName a.com\n\nHost bar baz\n  HostName b.com\n";
        assert_eq!(
            find_host_range(text, &["bar".to_string(), "baz".to_string()]),
            Some((3, 5))
        );
    }

    #[test]
    fn find_host_range_returns_none_when_no_match() {
        let text = "Host foo\n  HostName a.com\n";
        assert_eq!(find_host_range(text, &["missing".to_string()]), None);
    }

    #[test]
    fn render_host_block_includes_all_fields() {
        let block = render_host_block(
            &["github".to_string(), "gh".to_string()],
            Some("github.com"),
            Some("git"),
            Some("22"),
            Some("~/.ssh/id_ed25519"),
        );
        assert_eq!(
            block,
            "Host github gh\n  HostName github.com\n  User git\n  Port 22\n  IdentityFile ~/.ssh/id_ed25519"
        );
    }

    #[test]
    fn render_host_block_omits_none_fields() {
        let block = render_host_block(&["foo".to_string()], None, None, None, None);
        assert_eq!(block, "Host foo");
    }

    #[test]
    fn append_block_to_empty_text_has_no_leading_blank_line() {
        assert_eq!(append_block("", "Host foo\n  HostName a.com"), "Host foo\n  HostName a.com\n");
    }

    #[test]
    fn append_block_to_text_ending_with_newline_adds_blank_separator() {
        let existing = "Host existing\n  HostName e.com\n";
        let result = append_block(existing, "Host foo\n  HostName a.com");
        assert_eq!(
            result,
            "Host existing\n  HostName e.com\n\nHost foo\n  HostName a.com\n"
        );
    }

    #[test]
    fn append_block_to_text_missing_trailing_newline_still_separates_cleanly() {
        let existing = "Host existing\n  HostName e.com";
        let result = append_block(existing, "Host foo\n  HostName a.com");
        assert_eq!(
            result,
            "Host existing\n  HostName e.com\n\nHost foo\n  HostName a.com\n"
        );
    }

    #[test]
    fn replace_lines_only_touches_target_range() {
        let text = "# keep me\nHost foo\n  HostName old.com\n\nHost bar\n  HostName b.com\n";
        // "Host foo" block is lines 1..4 (Host foo, HostName old.com, blank)
        let result = replace_lines(text, (1, 4), "Host foo\n  HostName new.com");
        assert_eq!(
            result,
            "# keep me\nHost foo\n  HostName new.com\nHost bar\n  HostName b.com\n"
        );
    }

    #[test]
    fn remove_lines_only_touches_target_range() {
        let text = "# keep me\nHost foo\n  HostName old.com\n\nHost bar\n  HostName b.com\n";
        let result = remove_lines(text, (1, 4));
        assert_eq!(result, "# keep me\nHost bar\n  HostName b.com\n");
    }

    #[test]
    fn colliding_alias_detects_overlap() {
        let existing = vec![HostEntry {
            aliases: vec!["github".to_string()],
            host_name: None,
            user: None,
            port: None,
            identity_file: None,
        }];
        assert_eq!(
            colliding_alias(&["github".to_string()], &existing, None),
            Some("github".to_string())
        );
    }

    #[test]
    fn colliding_alias_ignores_excluded_entry() {
        let existing = vec![HostEntry {
            aliases: vec!["github".to_string()],
            host_name: None,
            user: None,
            port: None,
            identity_file: None,
        }];
        let exclude = vec!["github".to_string()];
        assert_eq!(
            colliding_alias(&["github".to_string()], &existing, Some(&exclude)),
            None
        );
    }

    #[test]
    fn colliding_alias_returns_none_when_no_overlap() {
        let existing = vec![HostEntry {
            aliases: vec!["gitlab".to_string()],
            host_name: None,
            user: None,
            port: None,
            identity_file: None,
        }];
        assert_eq!(colliding_alias(&["github".to_string()], &existing, None), None);
    }

    #[test]
    fn has_wildcard_alias_true_for_star_and_question_mark() {
        assert!(has_wildcard_alias(&["foo".to_string(), "ba*".to_string()]));
        assert!(has_wildcard_alias(&["fo?".to_string()]));
    }

    #[test]
    fn has_wildcard_alias_false_for_plain_alias() {
        assert!(!has_wildcard_alias(&["foo".to_string(), "bar".to_string()]));
    }
}
