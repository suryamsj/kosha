use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn tags_path() -> Result<PathBuf, String> {
    dirs::config_dir()
        .map(|dir| dir.join("kosha").join("tags.json"))
        .ok_or_else(|| "Could not determine config directory".to_string())
}

/// Applies a tag update to an in-memory tag map: sets `tags` for `key`, or
/// removes `key` entirely if `tags` is empty (keeps the file tidy instead
/// of accumulating empty-array entries).
fn apply_tag_update(
    mut all_tags: HashMap<String, Vec<String>>,
    key: String,
    tags: Vec<String>,
) -> HashMap<String, Vec<String>> {
    if tags.is_empty() {
        all_tags.remove(&key);
    } else {
        all_tags.insert(key, tags);
    }
    all_tags
}

#[tauri::command]
pub fn list_tags() -> Result<HashMap<String, Vec<String>>, String> {
    let path = tags_path()?;
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_tags(alias_key: String, tags: Vec<String>) -> Result<(), String> {
    let path = tags_path()?;
    let existing: HashMap<String, Vec<String>> = if path.exists() {
        let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&text).map_err(|e| e.to_string())?
    } else {
        HashMap::new()
    };

    let updated = apply_tag_update(existing, alias_key, tags);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&updated).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_tag_update_inserts_new_key() {
        let all_tags = HashMap::new();
        let result =
            apply_tag_update(all_tags, "github".to_string(), vec!["work".to_string()]);
        assert_eq!(result.get("github"), Some(&vec!["work".to_string()]));
    }

    #[test]
    fn apply_tag_update_overwrites_existing_key() {
        let mut all_tags = HashMap::new();
        all_tags.insert("github".to_string(), vec!["old".to_string()]);
        let result = apply_tag_update(
            all_tags,
            "github".to_string(),
            vec!["new".to_string(), "tags".to_string()],
        );
        assert_eq!(
            result.get("github"),
            Some(&vec!["new".to_string(), "tags".to_string()])
        );
    }

    #[test]
    fn apply_tag_update_removes_key_when_tags_empty() {
        let mut all_tags = HashMap::new();
        all_tags.insert("github".to_string(), vec!["work".to_string()]);
        let result = apply_tag_update(all_tags, "github".to_string(), vec![]);
        assert_eq!(result.get("github"), None);
        assert!(result.is_empty());
    }

    #[test]
    fn apply_tag_update_removing_absent_key_is_noop() {
        let all_tags = HashMap::new();
        let result = apply_tag_update(all_tags, "nonexistent".to_string(), vec![]);
        assert!(result.is_empty());
    }
}
