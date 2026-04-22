/// Ensures the opencode global config allows access to ~/.loom/**
/// so that sessions running inside ~/.loom/sessions/ are not blocked
/// by the external_directory permission.
///
/// The config file is JSONC (JSON with comments). We do targeted string
/// manipulation rather than a full parse to preserve comments and formatting.
use std::path::PathBuf;

/// Returns the path to the opencode global config file, or None if not found.
fn opencode_config_path() -> Option<PathBuf> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
        .map(PathBuf::from)?;

    let candidate = home.join(".config").join("opencode").join("opencode.json");
    if candidate.exists() {
        Some(candidate)
    } else {
        None
    }
}

/// Check whether the given raw config content already has an entry for ~/.loom.
fn has_loom_entry(content: &str) -> bool {
    content.contains("~/.loom") || content.contains("$HOME/.loom")
}

/// Inject `"~/.loom/**": "allow"` into the `external_directory` block.
/// If the block doesn't exist, create it inside `permission`.
/// If `permission` doesn't exist either, add both at the top level.
///
/// Returns the modified content, or None if we can't safely splice.
fn inject_loom_entry(content: &str) -> Option<String> {
    let new_entry = r#""~/.loom/**": "allow""#;

    // Try to find existing external_directory block and insert before its closing brace
    if let Some(pos) = content.find("\"external_directory\"") {
        // Find the opening brace of the block
        let after_key = &content[pos..];
        let brace_offset = after_key.find('{')? + pos;
        // Find the matching closing brace (simple single-level scan, no nesting expected here)
        let block_start = brace_offset + 1;
        let block_content = &content[block_start..];
        let close_offset = block_content.find('}')? + block_start;

        // Insert before the closing brace
        let before = &content[..close_offset];
        let after = &content[close_offset..];

        // Add a trailing comma to previous last entry if needed
        let trimmed = before.trim_end();
        let separator = if trimmed.ends_with('{') {
            "\n    ".to_string()
        } else {
            ",\n    ".to_string()
        };

        Some(format!("{}{}{}\n  {}", before, separator, new_entry, after))
    } else if let Some(pos) = content.find("\"permission\"") {
        // No external_directory block - find the permission object and add it
        let after_key = &content[pos..];
        let brace_offset = after_key.find('{')? + pos;
        let block_start = brace_offset + 1;
        let block_content = &content[block_start..];
        let close_offset = block_content.find('}')? + block_start;

        let before = &content[..close_offset];
        let after = &content[close_offset..];

        let trimmed = before.trim_end();
        let separator = if trimmed.ends_with('{') { "\n    " } else { ",\n    " };

        let new_block = format!(
            "{}{}\"external_directory\": {{\n      {}\n    }}\n  {}",
            before, separator, new_entry, after
        );
        Some(new_block)
    } else {
        // Can't safely find insertion point
        None
    }
}

/// Ensure ~/.loom/** is allowed in opencode's global config.
/// Writes atomically (temp file + rename) to avoid corruption.
pub fn ensure_loom_permitted() {
    let config_path = match opencode_config_path() {
        Some(p) => p,
        None => {
            tracing::debug!("opencode config not found, skipping permission check");
            return;
        }
    };

    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to read opencode config");
            return;
        }
    };

    if has_loom_entry(&content) {
        tracing::debug!("opencode config already allows ~/.loom, nothing to do");
        return;
    }

    tracing::info!("opencode config does not allow ~/.loom - patching...");

    let patched = match inject_loom_entry(&content) {
        Some(p) => p,
        None => {
            tracing::warn!(
                path = %config_path.display(),
                "Could not automatically patch opencode config - please add '\"~/.loom/**\": \"allow\"' under permission.external_directory manually"
            );
            return;
        }
    };

    // Atomic write: write to a temp file next to the config, then rename
    let tmp_path = config_path.with_extension("json.loom.tmp");
    if let Err(e) = std::fs::write(&tmp_path, &patched) {
        tracing::warn!(error = %e, "Failed to write temp opencode config");
        return;
    }
    if let Err(e) = std::fs::rename(&tmp_path, &config_path) {
        // Clean up temp on failure
        let _ = std::fs::remove_file(&tmp_path);
        tracing::warn!(error = %e, "Failed to rename temp opencode config");
        return;
    }

    tracing::info!(path = %config_path.display(), "Patched opencode config to allow ~/.loom/**");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_existing_loom_entry() {
        let content = r#"{ "permission": { "external_directory": { "~/.loom/**": "allow" } } }"#;
        assert!(has_loom_entry(content));
    }

    #[test]
    fn injects_into_existing_external_directory() {
        let content = r#"{
  "permission": {
    "external_directory": {
      "D:\\workspaces\\**": "allow"
    }
  }
}"#;
        let patched = inject_loom_entry(content).unwrap();
        assert!(patched.contains("~/.loom/**"));
        assert!(patched.contains("D:\\\\workspaces\\\\**"));
    }

    #[test]
    fn injects_when_no_external_directory() {
        let content = r#"{
  "permission": {
    "bash": "allow"
  }
}"#;
        let patched = inject_loom_entry(content).unwrap();
        assert!(patched.contains("external_directory"));
        assert!(patched.contains("~/.loom/**"));
    }
}
