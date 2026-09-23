use cotra_contracts::FailureCode;
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

const DEFAULT_MAX_READ_BYTES: usize = 1024 * 1024;
const DEFAULT_MAX_SEARCH_FILES: usize = 2_000;
const DEFAULT_MAX_SEARCH_RESULTS: usize = 200;
const MAX_SEARCH_FILE_BYTES: usize = 512 * 1024;

#[derive(Debug)]
pub struct ProviderError {
    pub code: FailureCode,
    pub message: String,
}

impl ProviderError {
    fn new(code: FailureCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    fn io(context: &str, error: io::Error) -> Self {
        Self::new(
            FailureCode::ProviderUnavailable,
            format!("{context}: {error}"),
        )
    }
}

#[derive(Debug, Clone)]
pub struct FsProvider {
    root: PathBuf,
    max_read_bytes: usize,
}

impl FsProvider {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, ProviderError> {
        let root = resolve_final_path(root.as_ref())
            .map_err(|error| ProviderError::io("resolve workspace root", error))?;
        if !root.is_dir() {
            return Err(ProviderError::new(
                FailureCode::WorkspaceDenied,
                "workspace root is not a directory",
            ));
        }
        Ok(Self {
            root,
            max_read_bytes: DEFAULT_MAX_READ_BYTES,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn stat(&self, relative: &str) -> Result<Value, ProviderError> {
        let resolved = self.resolve_existing(relative)?;
        let metadata =
            fs::metadata(&resolved).map_err(|error| ProviderError::io("stat target", error))?;
        Ok(json!({
            "path": relative,
            "kind": if metadata.is_dir() { "directory" } else if metadata.is_file() { "file" } else { "other" },
            "size": metadata.len(),
            "readonly": metadata.permissions().readonly()
        }))
    }

    pub fn list(&self, relative: &str) -> Result<Value, ProviderError> {
        let resolved = self.resolve_existing(relative)?;
        if !resolved.is_dir() {
            return Err(ProviderError::new(
                FailureCode::InvalidRequest,
                "list target is not a directory",
            ));
        }

        let mut entries = Vec::new();
        let iterator =
            fs::read_dir(&resolved).map_err(|error| ProviderError::io("list directory", error))?;
        for entry in iterator {
            let entry = entry.map_err(|error| ProviderError::io("read directory entry", error))?;
            let file_type = entry
                .file_type()
                .map_err(|error| ProviderError::io("read entry type", error))?;
            entries.push(json!({
                "name": entry.file_name().to_string_lossy(),
                "is_file": file_type.is_file(),
                "is_dir": file_type.is_dir(),
                "is_symlink": file_type.is_symlink()
            }));
        }
        entries.sort_by(|a, b| {
            a.get("name")
                .and_then(Value::as_str)
                .cmp(&b.get("name").and_then(Value::as_str))
        });

        Ok(json!({"path": relative, "entries": entries}))
    }

    pub fn read_text(&self, relative: &str) -> Result<Value, ProviderError> {
        let candidate = self.root.join(relative);
        let (file, resolved) = open_verified_file(&candidate)
            .map_err(|error| ProviderError::io("open file", error))?;
        self.ensure_within(&resolved)?;

        let metadata = file
            .metadata()
            .map_err(|error| ProviderError::io("read file metadata", error))?;
        if !metadata.is_file() {
            return Err(ProviderError::new(
                FailureCode::InvalidRequest,
                "read target is not a regular file",
            ));
        }
        if metadata.len() > self.max_read_bytes as u64 {
            return Err(ProviderError::new(
                FailureCode::OutputLimit,
                format!("file exceeds {} byte read limit", self.max_read_bytes),
            ));
        }

        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        file.take((self.max_read_bytes + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|error| ProviderError::io("read file", error))?;
        if bytes.len() > self.max_read_bytes {
            return Err(ProviderError::new(
                FailureCode::OutputLimit,
                "file exceeded read limit while reading",
            ));
        }
        let text = String::from_utf8(bytes).map_err(|_| {
            ProviderError::new(
                FailureCode::InvalidRequest,
                "SG-000001 fs.read supports UTF-8 text files only",
            )
        })?;

        Ok(json!({"path": relative, "text": text, "bytes": metadata.len()}))
    }

    pub fn search_text(
        &self,
        relative: &str,
        query: &str,
        requested_max_results: Option<usize>,
    ) -> Result<Value, ProviderError> {
        if query.is_empty() {
            return Err(ProviderError::new(
                FailureCode::InvalidRequest,
                "search query cannot be empty",
            ));
        }
        let max_results = requested_max_results
            .unwrap_or(DEFAULT_MAX_SEARCH_RESULTS)
            .clamp(1, DEFAULT_MAX_SEARCH_RESULTS);

        let start = self.resolve_existing(relative)?;
        let mut queue = VecDeque::new();
        queue.push_back(start);
        let mut scanned_files = 0usize;
        let mut matches = Vec::new();

        while let Some(path) = queue.pop_front() {
            if matches.len() >= max_results || scanned_files >= DEFAULT_MAX_SEARCH_FILES {
                break;
            }
            let metadata = match fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };
            if metadata.file_type().is_symlink() {
                continue;
            }

            if metadata.is_dir() {
                let entries = match fs::read_dir(&path) {
                    Ok(entries) => entries,
                    Err(_) => continue,
                };
                for entry in entries.flatten() {
                    let child = entry.path();
                    let file_type = match entry.file_type() {
                        Ok(file_type) => file_type,
                        Err(_) => continue,
                    };
                    if file_type.is_symlink() {
                        continue;
                    }
                    if file_type.is_dir() {
                        match resolve_final_path(&child) {
                            Ok(resolved_child) if is_within(&self.root, &resolved_child) => {
                                queue.push_back(resolved_child);
                            }
                            _ => {}
                        }
                    } else if file_type.is_file() {
                        queue.push_back(child);
                    }
                }
                continue;
            }

            if !metadata.is_file() {
                continue;
            }

            scanned_files += 1;
            if metadata.len() > MAX_SEARCH_FILE_BYTES as u64 {
                continue;
            }

            let (file, resolved) = match open_verified_file(&path) {
                Ok(opened) => opened,
                Err(_) => continue,
            };
            if !is_within(&self.root, &resolved) {
                continue;
            }

            let mut bytes = Vec::with_capacity(metadata.len() as usize);
            if file
                .take((MAX_SEARCH_FILE_BYTES + 1) as u64)
                .read_to_end(&mut bytes)
                .is_err()
            {
                continue;
            }
            if bytes.len() > MAX_SEARCH_FILE_BYTES {
                continue;
            }
            let text = match String::from_utf8(bytes) {
                Ok(text) => text,
                Err(_) => continue,
            };

            for (index, line) in text.lines().enumerate() {
                if line.contains(query) {
                    let rel = resolved
                        .strip_prefix(&self.root)
                        .unwrap_or(&resolved)
                        .to_string_lossy()
                        .to_string();
                    matches.push(json!({
                        "path": rel,
                        "line": index + 1,
                        "text": truncate_line(line, 400)
                    }));
                    if matches.len() >= max_results {
                        break;
                    }
                }
            }
        }

        let truncated = matches.len() >= max_results || scanned_files >= DEFAULT_MAX_SEARCH_FILES;
        Ok(json!({
            "path": relative,
            "query": query,
            "matches": matches,
            "scanned_files": scanned_files,
            "truncated": truncated
        }))
    }

    fn resolve_existing(&self, relative: &str) -> Result<PathBuf, ProviderError> {
        let candidate = self.root.join(relative);
        let resolved = resolve_final_path(&candidate)
            .map_err(|error| ProviderError::io("resolve target", error))?;
        self.ensure_within(&resolved)?;
        Ok(resolved)
    }

    fn ensure_within(&self, resolved: &Path) -> Result<(), ProviderError> {
        if !is_within(&self.root, resolved) {
            return Err(ProviderError::new(
                FailureCode::PathEscape,
                "resolved path is outside the trusted workspace",
            ));
        }
        Ok(())
    }
}

fn truncate_line(line: &str, max_chars: usize) -> String {
    let mut chars = line.chars();
    let mut output: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        output.push('…');
    }
    output
}

fn open_verified_file(path: &Path) -> io::Result<(File, PathBuf)> {
    let file = File::open(path)?;
    #[cfg(windows)]
    let resolved = {
        use std::os::windows::io::AsRawHandle;
        win::final_path_from_handle(file.as_raw_handle())
    }?;
    #[cfg(not(windows))]
    let resolved = fs::canonicalize(path)?;
    Ok((file, resolved))
}

fn resolve_final_path(path: &Path) -> io::Result<PathBuf> {
    #[cfg(windows)]
    {
        win::final_path(path)
    }
    #[cfg(not(windows))]
    {
        fs::canonicalize(path)
    }
}

#[cfg(windows)]
fn is_within(root: &Path, candidate: &Path) -> bool {
    fn key(path: &Path) -> String {
        path.to_string_lossy()
            .replace('/', "\\")
            .trim_end_matches('\\')
            .to_lowercase()
    }
    let root = key(root);
    let candidate = key(candidate);
    candidate == root || candidate.starts_with(&(root + "\\"))
}

#[cfg(not(windows))]
fn is_within(root: &Path, candidate: &Path) -> bool {
    candidate == root || candidate.starts_with(root)
}

#[cfg(windows)]
mod win {
    use std::ffi::c_void;
    use std::io;
    use std::os::windows::ffi::OsStrExt;
    use std::path::{Path, PathBuf};

    type Handle = *mut c_void;

    const FILE_READ_ATTRIBUTES: u32 = 0x0080;
    const FILE_SHARE_READ: u32 = 0x00000001;
    const FILE_SHARE_WRITE: u32 = 0x00000002;
    const FILE_SHARE_DELETE: u32 = 0x00000004;
    const OPEN_EXISTING: u32 = 3;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x02000000;
    const INVALID_HANDLE_VALUE: Handle = -1isize as Handle;

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateFileW(
            file_name: *const u16,
            desired_access: u32,
            share_mode: u32,
            security_attributes: *const c_void,
            creation_disposition: u32,
            flags_and_attributes: u32,
            template_file: Handle,
        ) -> Handle;
        fn GetFinalPathNameByHandleW(
            file: Handle,
            file_path: *mut u16,
            file_path_len: u32,
            flags: u32,
        ) -> u32;
        fn CloseHandle(handle: Handle) -> i32;
    }

    pub fn final_path(path: &Path) -> io::Result<PathBuf> {
        let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
        wide.push(0);
        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                FILE_READ_ATTRIBUTES,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                std::ptr::null(),
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS,
                std::ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let result = final_path_from_handle_impl(handle);
        unsafe {
            CloseHandle(handle);
        }
        result
    }

    pub fn final_path_from_handle(
        raw_handle: std::os::windows::io::RawHandle,
    ) -> io::Result<PathBuf> {
        final_path_from_handle_impl(raw_handle.cast())
    }

    fn final_path_from_handle_impl(handle: Handle) -> io::Result<PathBuf> {
        let mut buffer = vec![0u16; 512];
        loop {
            let length = unsafe {
                GetFinalPathNameByHandleW(handle, buffer.as_mut_ptr(), buffer.len() as u32, 0)
            };
            if length == 0 {
                return Err(io::Error::last_os_error());
            }
            if length < buffer.len() as u32 {
                let text = String::from_utf16_lossy(&buffer[..length as usize]);
                return Ok(PathBuf::from(strip_extended_prefix(&text)));
            }
            buffer.resize(length as usize + 1, 0);
        }
    }

    fn strip_extended_prefix(path: &str) -> String {
        if let Some(rest) = path.strip_prefix(r"\\?\UNC\") {
            return format!(r"\\{rest}");
        }
        if let Some(rest) = path.strip_prefix(r"\\?\") {
            return rest.to_owned();
        }
        path.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("cotra-fs-{label}-{suffix}"));
        std::fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    #[test]
    fn reads_utf8_file_inside_workspace() {
        let root = temp_dir("read");
        std::fs::write(root.join("hello.txt"), "hello cotra").expect("write");
        let provider = FsProvider::new(&root).expect("provider");
        let result = provider.read_text("hello.txt").expect("read");
        assert_eq!(result["text"], "hello cotra");
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_escape() {
        use std::os::unix::fs::symlink;
        let root = temp_dir("root");
        let outside = temp_dir("outside");
        std::fs::write(outside.join("secret.txt"), "secret").expect("write");
        symlink(&outside, root.join("escape")).expect("symlink");
        let provider = FsProvider::new(&root).expect("provider");
        let error = provider
            .read_text("escape/secret.txt")
            .expect_err("escape must fail");
        assert_eq!(error.code, FailureCode::PathEscape);
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(outside);
    }

    #[cfg(windows)]
    #[test]
    fn rejects_junction_escape() {
        use std::process::Command;
        let root = temp_dir("root");
        let outside = temp_dir("outside");
        std::fs::write(outside.join("secret.txt"), "secret").expect("write");
        let junction = root.join("escape");
        let status = Command::new("cmd")
            .arg("/C")
            .arg("mklink")
            .arg("/J")
            .arg(&junction)
            .arg(&outside)
            .status()
            .expect("mklink");
        assert!(status.success(), "junction fixture creation failed");
        let provider = FsProvider::new(&root).expect("provider");
        let error = provider
            .read_text("escape/secret.txt")
            .expect_err("junction escape must fail");
        assert_eq!(error.code, FailureCode::PathEscape);
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(outside);
    }

    #[test]
    fn search_is_bounded_and_does_not_follow_links() {
        let root = temp_dir("search");
        std::fs::write(root.join("a.txt"), "one\nneedle\nthree").expect("write");
        let provider = FsProvider::new(&root).expect("provider");
        let result = provider
            .search_text(".", "needle", Some(5))
            .expect("search");
        assert_eq!(result["matches"].as_array().expect("matches").len(), 1);
        let _ = std::fs::remove_dir_all(root);
    }
}
