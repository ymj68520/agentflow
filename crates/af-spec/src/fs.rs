//! Filesystem tooling: canonical JSON loading, file digests, path safety and
//! the `spec/idl` walker.

use crate::error::SpecError;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Reads and parses a JSON file that must be in **canonical registry form**:
/// UTF-8, LF-only, final newline, 2-space indent and lexicographically sorted
/// object keys at every level. The last property is verified by round-tripping
/// the parsed value through a key-sorting pretty printer and comparing bytes.
pub fn load_canonical_json(root: &Path, relative: &str) -> Result<serde_json::Value, SpecError> {
    let path = root.join(relative);
    let display = relative.to_string();
    let bytes = std::fs::read(&path).map_err(|e| SpecError::Io {
        path: display.clone(),
        detail: e.to_string(),
    })?;
    if bytes.contains(&b'\r') {
        return Err(SpecError::CanonicalForm {
            path: display,
            detail: "file contains CR bytes; registry JSON uses LF only".into(),
        });
    }
    if !bytes.ends_with(b"\n") {
        return Err(SpecError::CanonicalForm {
            path: display,
            detail: "file does not end with a newline".into(),
        });
    }
    let text = std::str::from_utf8(&bytes).map_err(|e| SpecError::Json {
        path: display.clone(),
        detail: e.to_string(),
    })?;
    let value: serde_json::Value = serde_json::from_str(text).map_err(|e| SpecError::Json {
        path: display.clone(),
        detail: e.to_string(),
    })?;
    let expected = serde_json::to_string_pretty(&value).map_err(|e| SpecError::Json {
        path: display.clone(),
        detail: e.to_string(),
    })?;
    let actual = text.strip_suffix('\n').unwrap_or(text);
    if actual != expected {
        // The canonical form is exactly what a key-sorting pretty printer
        // emits: sorted object keys at every level, 2-space indent, no
        // insignificant whitespace. Any byte difference is a violation.
        return Err(SpecError::CanonicalForm {
            path: display,
            detail: "bytes do not round-trip through the canonical pretty printer \
                     (keys must be lexicographically sorted, 2-space indent)"
                .into(),
        });
    }
    Ok(value)
}

/// Deserializes a typed model from an already-loaded JSON value.
pub fn from_value<T: serde::de::DeserializeOwned>(
    value: serde_json::Value,
    path: &str,
) -> Result<T, SpecError> {
    serde_json::from_value(value).map_err(|e| SpecError::Json {
        path: path.to_string(),
        detail: e.to_string(),
    })
}

/// Reads a plain (non-registry-form) JSON file — used for the pre-existing
/// closed tables and the conformance corpus, which keep their own frozen
/// formatting.
pub fn load_plain_json(root: &Path, relative: &str) -> Result<serde_json::Value, SpecError> {
    let path = root.join(relative);
    let display = relative.to_string();
    let text = std::fs::read_to_string(&path).map_err(|e| SpecError::Io {
        path: display.clone(),
        detail: e.to_string(),
    })?;
    serde_json::from_str(&text).map_err(|e| SpecError::Json {
        path: display,
        detail: e.to_string(),
    })
}

/// Reads a UTF-8 file.
pub fn read_file(root: &Path, relative: &str) -> Result<Vec<u8>, SpecError> {
    std::fs::read(root.join(relative)).map_err(|e| SpecError::Io {
        path: relative.to_string(),
        detail: e.to_string(),
    })
}

/// `SHA256(file bytes)`, lowercase hex — the `ArtifactFileDigest` algorithm.
/// This is repository tooling identity, never a HashV1 semantic digest.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for b in digest {
        write!(out, "{b:02x}").expect("String formatting is infallible");
    }
    out
}

/// Validates a normative-source logical path and resolves it under `root`.
///
/// Paths are repo-relative, `/`-separated, without `..`, backslashes, drive
/// letters or empty segments.
pub fn safe_join(root: &Path, logical: &str) -> Result<PathBuf, SpecError> {
    let reject = |detail: &str| SpecError::InvalidSourcePath {
        path: logical.to_string(),
        detail: detail.to_string(),
    };
    if logical.is_empty() {
        return Err(reject("path is empty"));
    }
    if logical.starts_with('/') || logical.contains(':') {
        return Err(reject("path must be relative"));
    }
    if logical.contains('\\') {
        return Err(reject("path must use '/' separators"));
    }
    if logical
        .split('/')
        .any(|seg| seg.is_empty() || seg == "." || seg == "..")
    {
        return Err(reject("path segments must be non-empty and not traversal"));
    }
    Ok(root.join(logical))
}

/// Collects every `*.afidl` file under `dir`, recursively, sorted by path.
pub fn collect_afidl_files(root: &Path, dir: &str) -> Vec<String> {
    let base = root.join(dir);
    let mut out = Vec::new();
    let mut stack = vec![base];
    while let Some(current) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("afidl") {
                if let Some(rel) = path.strip_prefix(root).ok().and_then(|p| p.to_str()) {
                    out.push(rel.replace('\\', "/"));
                }
            }
        }
    }
    out.sort();
    out
}

/// Collects every `*.json` file under `dir`, recursively, sorted by path.
/// Missing directories yield an empty list (optional conformance roots).
pub fn collect_json_files(root: &Path, dir: &str) -> Vec<String> {
    let base = root.join(dir);
    if !base.is_dir() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut stack = vec![base];
    while let Some(current) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Some(rel) = path.strip_prefix(root).ok().and_then(|p| p.to_str()) {
                    out.push(rel.replace('\\', "/"));
                }
            }
        }
    }
    out.sort();
    out
}
