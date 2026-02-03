use std::path::{Path, PathBuf};

/// Validates that a path is safe and within expected boundaries.
/// This prevents path-injection attacks where user-controlled input
/// could escape intended directories via `..` traversal.
///
/// # Arguments
/// * `path` - The path to validate
/// * `allowed_base` - Optional base directory the path must be within
///
/// # Returns
/// * `Ok(PathBuf)` - The canonicalized, validated path
/// * `Err(String)` - Error message if validation fails
///
/// # Security
/// This function:
/// 1. Rejects paths containing `..` or null bytes
/// 2. Canonicalizes the path to resolve symlinks
/// 3. Validates the resulting path is within the allowed base directory
pub fn validate_path<P: AsRef<Path>>(path: P, allowed_base: Option<&Path>) -> Result<PathBuf, String> {
    let path = path.as_ref();

    // Check for null bytes (can be used to truncate paths in some languages)
    if path.to_string_lossy().contains('\0') {
        return Err("Path contains null bytes".to_string());
    }

    // Check for explicit path traversal in the original input
    let path_str = path.to_string_lossy();
    if path_str.contains("..") {
        return Err("Path traversal detected (contains '..')".to_string());
    }

    // For validation against a base directory
    if let Some(base) = allowed_base {
        // Canonicalize both paths to resolve symlinks and normalize
        let canonical_base = base.canonicalize()
            .map_err(|e| format!("Failed to canonicalize base path: {}", e))?;

        // If the path doesn't exist yet, we validate the parent directory
        let canonical_path = if path.exists() {
            path.canonicalize()
                .map_err(|e| format!("Failed to canonicalize path: {}", e))?
        } else {
            // For non-existent paths, canonicalize the parent and append the filename
            if let Some(parent) = path.parent() {
                if parent.exists() {
                    let canonical_parent = parent.canonicalize()
                        .map_err(|e| format!("Failed to canonicalize parent path: {}", e))?;
                    if let Some(filename) = path.file_name() {
                        canonical_parent.join(filename)
                    } else {
                        return Err("Path has no filename component".to_string());
                    }
                } else {
                    // Parent doesn't exist - just use the path as-is for validation
                    path.to_path_buf()
                }
            } else {
                path.to_path_buf()
            }
        };

        // Verify the path is within the allowed base
        if !canonical_path.starts_with(&canonical_base) {
            return Err(format!(
                "Path escapes allowed directory: {:?} is not within {:?}",
                canonical_path, canonical_base
            ));
        }

        Ok(canonical_path)
    } else {
        // No base directory constraint - just canonicalize if exists
        if path.exists() {
            path.canonicalize()
                .map_err(|e| format!("Failed to canonicalize path: {}", e))
        } else {
            Ok(path.to_path_buf())
        }
    }
}

/// Validates a path is within a data directory.
/// Convenience wrapper for common case of validating paths within app data.
pub fn validate_data_path<P: AsRef<Path>>(path: P, data_dir: &Path) -> Result<PathBuf, String> {
    validate_path(path, Some(data_dir))
}

/// Validates that a user-provided path string is safe.
/// Does basic sanitization before converting to PathBuf.
pub fn sanitize_path_string(path_str: &str) -> Result<PathBuf, String> {
    if path_str.is_empty() {
        return Err("Path cannot be empty".to_string());
    }

    if path_str.contains('\0') {
        return Err("Path contains null bytes".to_string());
    }

    // Check for path traversal
    if path_str.contains("..") {
        return Err("Path traversal detected (contains '..')".to_string());
    }

    // Reject some dangerous patterns (Unix and Windows)
    if path_str.starts_with('/') || path_str.contains(":\\") || path_str.starts_with("\\\\") {
        // Absolute paths are allowed, but we log for auditing
        tracing::debug!("Processing absolute path: {}", path_str);
    }

    Ok(PathBuf::from(path_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_path_traversal_rejected() {
        let result = validate_path("../etc/passwd", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Path traversal"));
    }

    #[test]
    fn test_null_byte_rejected() {
        let result = validate_path("foo\0bar", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("null bytes"));
    }

    #[test]
    fn test_valid_path_within_base() {
        let tmp = tempdir().unwrap();
        let valid_file = tmp.path().join("test.txt");
        fs::write(&valid_file, "test").unwrap();

        let result = validate_path(&valid_file, Some(tmp.path()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_escapes_base_rejected() {
        let tmp = tempdir().unwrap();
        let base = tmp.path().join("allowed");
        fs::create_dir_all(&base).unwrap();

        // Try to access a file outside the allowed directory
        let outside = tmp.path().join("outside.txt");
        fs::write(&outside, "test").unwrap();

        let result = validate_path(&outside, Some(&base));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("escapes allowed directory"));
    }
}
