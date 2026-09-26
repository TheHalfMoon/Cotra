use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use cotra_contracts::{FailureCode, RequestEnvelope, Workspace};
use serde_json::Value;

pub const POLICY_REVISION: &str = "sg-000010-v1";
const MAX_PROCESS_ARGV_ITEMS: usize = 64;
const MAX_PROCESS_ARG_UTF16: usize = 8192;
const MAX_PROCESS_COMMAND_UTF16: usize = 30_000;
const MAX_PROCESS_TIMEOUT_MS: u64 = 30 * 60 * 1000;
const MAX_PROCESS_STDOUT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_PROCESS_STDERR_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyError {
    pub code: FailureCode,
    pub message: String,
}

impl PolicyError {
    fn new(code: FailureCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PolicyEngine {
    workspaces: BTreeMap<String, Workspace>,
}

impl PolicyEngine {
    pub fn new(workspaces: Vec<Workspace>) -> Result<Self, PolicyError> {
        let mut map = BTreeMap::new();
        for workspace in workspaces {
            let root = fs::canonicalize(&workspace.root).map_err(|error| {
                PolicyError::new(
                    FailureCode::WorkspaceDenied,
                    format!(
                        "workspace root could not be resolved for {}: {error}",
                        workspace.id
                    ),
                )
            })?;
            if !root.is_dir() {
                return Err(PolicyError::new(
                    FailureCode::WorkspaceDenied,
                    format!("workspace root is not a directory: {}", root.display()),
                ));
            }
            map.insert(
                workspace.id.clone(),
                Workspace {
                    id: workspace.id,
                    root,
                },
            );
        }
        Ok(Self { workspaces: map })
    }

    pub fn workspace(&self, workspace_id: &str) -> Result<&Workspace, PolicyError> {
        self.workspaces.get(workspace_id).ok_or_else(|| {
            PolicyError::new(
                FailureCode::WorkspaceDenied,
                format!("workspace is not configured: {workspace_id}"),
            )
        })
    }

    pub fn authorize(&self, request: &RequestEnvelope) -> Result<(), PolicyError> {
        let workspace = self.workspace(&request.workspace_id)?;

        match (request.capability.as_str(), request.operation.as_str()) {
            ("system.status", "get") | ("workspace.get", "get") => Ok(()),
            ("fs.stat", "stat") | ("fs.list", "list") | ("fs.read", "read") => {
                validate_target(request.target.as_deref())?;
                Ok(())
            }
            ("fs.search", "search") => {
                validate_target(request.target.as_deref())?;
                if request
                    .arguments
                    .get("query")
                    .and_then(Value::as_str)
                    .is_none_or(str::is_empty)
                {
                    return Err(PolicyError::new(
                        FailureCode::InvalidRequest,
                        "fs.search requires a non-empty query",
                    ));
                }
                Ok(())
            }
            ("fs.write", "preview") => {
                validate_target(request.target.as_deref())?;
                require_content(request)?;
                Ok(())
            }
            ("fs.write", "write") => {
                validate_target(request.target.as_deref())?;
                require_content(request)?;
                Ok(())
            }
            ("git.status", "status") | ("git.diff", "diff") | ("git.log", "log") => {
                validate_target(request.target.as_deref())?;
                Ok(())
            }
            ("process.spawn", "spawn") => validate_process_spawn(workspace, request),
            _ => Err(PolicyError::new(
                FailureCode::CapabilityDenied,
                format!(
                    "capability is not allowed in policy revision {POLICY_REVISION}: {}:{}",
                    request.capability, request.operation
                ),
            )),
        }
    }
}

fn validate_process_spawn(
    workspace: &Workspace,
    request: &RequestEnvelope,
) -> Result<(), PolicyError> {
    if request.target.is_some() {
        return Err(PolicyError::new(
            FailureCode::InvalidRequest,
            "process.spawn does not accept a target field",
        ));
    }
    const EXPECTED_KEYS: &[&str] = &[
        "executable",
        "argv",
        "cwd",
        "timeout_ms",
        "stdout_bytes",
        "stderr_bytes",
        "stdin_policy",
        "network_class",
    ];
    for key in request.arguments.keys() {
        if !EXPECTED_KEYS.contains(&key.as_str()) {
            return Err(PolicyError::new(
                FailureCode::InvalidRequest,
                format!("process.spawn does not accept arguments.{key}"),
            ));
        }
    }

    let executable = request
        .arguments
        .get("executable")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            PolicyError::new(
                FailureCode::InvalidRequest,
                "process.spawn requires arguments.executable",
            )
        })?;
    if executable.is_empty() || executable.contains('\0') || executable.encode_utf16().count() > 1024
    {
        return Err(PolicyError::new(
            FailureCode::InvalidRequest,
            "process.spawn executable is invalid",
        ));
    }
    let executable = Path::new(executable);
    if !executable.is_absolute() {
        return Err(PolicyError::new(
            FailureCode::InvalidRequest,
            "process.spawn executable must be absolute",
        ));
    }
    enforce_sg000010_executable_policy(executable)?;

    let argv = request
        .arguments
        .get("argv")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            PolicyError::new(
                FailureCode::InvalidRequest,
                "process.spawn requires arguments.argv",
            )
        })?;
    if argv.len() > MAX_PROCESS_ARGV_ITEMS {
        return Err(PolicyError::new(
            FailureCode::InvalidRequest,
            "process.spawn argv exceeds the item limit",
        ));
    }
    let mut command_utf16 = executable.to_string_lossy().encode_utf16().count() + 2;
    for value in argv {
        let argument = value.as_str().ok_or_else(|| {
            PolicyError::new(
                FailureCode::InvalidRequest,
                "process.spawn argv values must be strings",
            )
        })?;
        if argument.contains('\0') || argument.encode_utf16().count() > MAX_PROCESS_ARG_UTF16 {
            return Err(PolicyError::new(
                FailureCode::InvalidRequest,
                "process.spawn argv contains an invalid argument",
            ));
        }
        command_utf16 = command_utf16
            .saturating_add(argument.encode_utf16().count().saturating_mul(2))
            .saturating_add(4);
        if command_utf16 > MAX_PROCESS_COMMAND_UTF16 {
            return Err(PolicyError::new(
                FailureCode::InvalidRequest,
                "process.spawn command line exceeds the Windows bound",
            ));
        }
    }

    let cwd = request
        .arguments
        .get("cwd")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            PolicyError::new(
                FailureCode::InvalidRequest,
                "process.spawn requires arguments.cwd",
            )
        })?;
    if cwd.is_empty() || cwd.contains('\0') || cwd.encode_utf16().count() > 1024 {
        return Err(PolicyError::new(
            FailureCode::InvalidRequest,
            "process.spawn cwd is invalid",
        ));
    }
    validate_relative_target(cwd)?;

    bounded_u64(
        request,
        "timeout_ms",
        1000,
        MAX_PROCESS_TIMEOUT_MS,
        "process.spawn timeout",
    )?;
    bounded_u64(
        request,
        "stdout_bytes",
        1,
        MAX_PROCESS_STDOUT_BYTES,
        "process.spawn stdout",
    )?;
    bounded_u64(
        request,
        "stderr_bytes",
        1,
        MAX_PROCESS_STDERR_BYTES,
        "process.spawn stderr",
    )?;

    if request
        .arguments
        .get("stdin_policy")
        .and_then(Value::as_str)
        != Some("null")
    {
        return Err(PolicyError::new(
            FailureCode::InvalidRequest,
            "process.spawn stdin_policy must be null",
        ));
    }
    if request
        .arguments
        .get("network_class")
        .and_then(Value::as_str)
        != Some("NONE")
    {
        return Err(PolicyError::new(
            FailureCode::CapabilityDenied,
            "process.spawn network_class must be NONE",
        ));
    }

    // Ensure the configured root still exists/canonicalizes at authorization time.
    fs::canonicalize(&workspace.root).map_err(|error| {
        PolicyError::new(
            FailureCode::WorkspaceDenied,
            format!(
                "workspace root could not be resolved for {}: {error}",
                workspace.id
            ),
        )
    })?;
    Ok(())
}

#[cfg(windows)]
fn enforce_sg000010_executable_policy(executable: &Path) -> Result<(), PolicyError> {
    let system_root = std::env::var_os("SystemRoot").ok_or_else(|| {
        PolicyError::new(
            FailureCode::CapabilityDenied,
            "SystemRoot is unavailable for the SG-000010 executable policy",
        )
    })?;
    let allowed = fs::canonicalize(Path::new(&system_root).join("System32").join("whoami.exe"))
        .map_err(|error| {
            PolicyError::new(
                FailureCode::CapabilityDenied,
                format!("resolve SG-000010 executable policy target: {error}"),
            )
        })?;
    let requested = fs::canonicalize(executable).map_err(|error| {
        PolicyError::new(
            FailureCode::InvalidRequest,
            format!("resolve process.spawn executable: {error}"),
        )
    })?;
    if requested != allowed {
        return Err(PolicyError::new(
            FailureCode::CapabilityDenied,
            "process.spawn executable is not in the SG-000010 qualified executable policy",
        ));
    }
    Ok(())
}

#[cfg(not(windows))]
fn enforce_sg000010_executable_policy(executable: &Path) -> Result<(), PolicyError> {
    let requested = fs::canonicalize(executable).map_err(|error| {
        PolicyError::new(
            FailureCode::InvalidRequest,
            format!("resolve process.spawn executable: {error}"),
        )
    })?;
    let fixture = fs::canonicalize(std::env::current_exe().map_err(|error| {
        PolicyError::new(
            FailureCode::CapabilityDenied,
            format!("resolve SG-000010 non-Windows fixture executable: {error}"),
        )
    })?)
    .map_err(|error| {
        PolicyError::new(
            FailureCode::CapabilityDenied,
            format!("canonicalize SG-000010 non-Windows fixture executable: {error}"),
        )
    })?;
    if requested != fixture {
        return Err(PolicyError::new(
            FailureCode::CapabilityDenied,
            "process.spawn executable is not in the SG-000010 qualified executable policy",
        ));
    }
    Ok(())
}

fn bounded_u64(
    request: &RequestEnvelope,
    key: &str,
    minimum: u64,
    maximum: u64,
    label: &str,
) -> Result<u64, PolicyError> {
    let value = request
        .arguments
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            PolicyError::new(
                FailureCode::InvalidRequest,
                format!("{label} requires an unsigned integer"),
            )
        })?;
    if value < minimum || value > maximum {
        return Err(PolicyError::new(
            FailureCode::InvalidRequest,
            format!("{label} is outside the allowed bound"),
        ));
    }
    Ok(value)
}

fn require_content(request: &RequestEnvelope) -> Result<(), PolicyError> {
    if request
        .arguments
        .get("content")
        .and_then(Value::as_str)
        .is_none()
    {
        return Err(PolicyError::new(
            FailureCode::InvalidRequest,
            "fs.write requires string content",
        ));
    }
    Ok(())
}

fn validate_target(target: Option<&str>) -> Result<(), PolicyError> {
    let target = target.ok_or_else(|| {
        PolicyError::new(
            FailureCode::InvalidRequest,
            "this capability requires a relative target path",
        )
    })?;
    validate_relative_target(target)
}

fn validate_relative_target(target: &str) -> Result<(), PolicyError> {
    if target.contains('\0') {
        return Err(PolicyError::new(
            FailureCode::InvalidRequest,
            "target contains a NUL byte",
        ));
    }

    let path = Path::new(target);
    if path.is_absolute() || target.starts_with(['/', '\\']) {
        return Err(PolicyError::new(
            FailureCode::PathEscape,
            "absolute paths are not allowed",
        ));
    }

    let bytes = target.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return Err(PolicyError::new(
            FailureCode::PathEscape,
            "drive-qualified paths are not allowed",
        ));
    }
    if target.starts_with("\\\\?\\") || target.starts_with("\\\\.\\") {
        return Err(PolicyError::new(
            FailureCode::PathEscape,
            "Windows device paths are not allowed",
        ));
    }

    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(PolicyError::new(
                    FailureCode::PathEscape,
                    "parent/root/prefix path components are not allowed",
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cotra_contracts::RequestEnvelope;
    use serde_json::json;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEMP_ROOT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn temp_root() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let sequence = TEMP_ROOT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "cotra-policy-{}-{suffix}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("create temp workspace");
        root
    }

    fn request(target: &str) -> RequestEnvelope {
        RequestEnvelope {
            version: 1,
            request_id: "r1".into(),
            client_session_id: "s1".into(),
            workspace_id: "default".into(),
            capability: "fs.read".into(),
            operation: "read".into(),
            target: Some(target.into()),
            arguments: json!({}),
        }
    }

    fn process_request(executable: &Path) -> RequestEnvelope {
        RequestEnvelope {
            version: 1,
            request_id: "process-r1".into(),
            client_session_id: "s1".into(),
            workspace_id: "default".into(),
            capability: "process.spawn".into(),
            operation: "spawn".into(),
            target: None,
            arguments: json!({
                "executable": executable.to_string_lossy(),
                "argv": ["--version"],
                "cwd": ".",
                "timeout_ms": 30_000,
                "stdout_bytes": 2 * 1024 * 1024,
                "stderr_bytes": 256 * 1024,
                "stdin_policy": "null",
                "network_class": "NONE"
            }),
        }
    }

    #[cfg(windows)]
    fn process_fixture_executable() -> PathBuf {
        PathBuf::from(std::env::var_os("SystemRoot").expect("SystemRoot"))
            .join("System32")
            .join("whoami.exe")
    }

    #[cfg(not(windows))]
    fn process_fixture_executable() -> PathBuf {
        std::env::current_exe().expect("current executable")
    }

    fn engine(root: &Path) -> PolicyEngine {
        PolicyEngine::new(vec![Workspace {
            id: "default".into(),
            root: root.to_path_buf(),
        }])
        .expect("policy")
    }

    #[test]
    fn rejects_parent_escape() {
        let root = temp_root();
        let error = engine(&root)
            .authorize(&request("../outside.txt"))
            .expect_err("escape must fail");
        assert_eq!(error.code, FailureCode::PathEscape);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_windows_root_and_device_paths_portably() {
        assert!(validate_relative_target(r"C:\Windows\System32").is_err());
        assert!(validate_relative_target(r"\Windows\System32").is_err());
        assert!(validate_relative_target(r"\\server\share\file").is_err());
        assert!(validate_relative_target(r"\\?\C:\Windows").is_err());
        assert!(validate_relative_target(r"\\.\PhysicalDrive0").is_err());
    }

    #[test]
    fn denies_unknown_capability() {
        let root = temp_root();
        let mut req = request("file.txt");
        req.capability = "process.kill".into();
        req.operation = "kill".into();
        let error = engine(&root).authorize(&req).expect_err("must be denied");
        assert_eq!(error.code, FailureCode::CapabilityDenied);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn write_requires_content() {
        let root = temp_root();
        let mut req = request("file.txt");
        req.capability = "fs.write".into();
        req.operation = "write".into();
        let error = engine(&root)
            .authorize(&req)
            .expect_err("missing content must fail");
        assert_eq!(error.code, FailureCode::InvalidRequest);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn authorizes_only_bounded_argv_process_spawn() {
        let root = temp_root();
        let executable = process_fixture_executable();
        let req = process_request(&executable);
        engine(&root)
            .authorize(&req)
            .expect("bounded process.spawn");

        let mut wrong_operation = req.clone();
        wrong_operation.operation = "run".into();
        assert_eq!(
            engine(&root).authorize(&wrong_operation).unwrap_err().code,
            FailureCode::CapabilityDenied
        );

        let mut powershell = req.clone();
        powershell.capability = "powershell.run".into();
        powershell.operation = "run".into();
        assert_eq!(
            engine(&root).authorize(&powershell).unwrap_err().code,
            FailureCode::CapabilityDenied
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn process_spawn_rejects_escape_shell_network_stdin_env_and_unbounded_inputs() {
        let root = temp_root();
        let executable = process_fixture_executable();
        let base = process_request(&executable);

        let mut relative_executable = base.clone();
        relative_executable.arguments["executable"] = json!("tool.exe");
        assert_eq!(
            engine(&root)
                .authorize(&relative_executable)
                .unwrap_err()
                .code,
            FailureCode::InvalidRequest
        );

        let mut cwd_escape = base.clone();
        cwd_escape.arguments["cwd"] = json!("..");
        assert_eq!(
            engine(&root).authorize(&cwd_escape).unwrap_err().code,
            FailureCode::PathEscape
        );

        let mut nul_argv = base.clone();
        nul_argv.arguments["argv"] = json!(["bad\0arg"]);
        assert_eq!(
            engine(&root).authorize(&nul_argv).unwrap_err().code,
            FailureCode::InvalidRequest
        );

        let mut network = base.clone();
        network.arguments["network_class"] = json!("DIRECT_DESTINATION");
        assert_eq!(
            engine(&root).authorize(&network).unwrap_err().code,
            FailureCode::CapabilityDenied
        );

        let mut stdin = base.clone();
        stdin.arguments["stdin_policy"] = json!("text");
        assert_eq!(
            engine(&root).authorize(&stdin).unwrap_err().code,
            FailureCode::InvalidRequest
        );

        let mut env = base.clone();
        env.arguments["env"] = json!({"SECRET": "x"});
        assert_eq!(
            engine(&root).authorize(&env).unwrap_err().code,
            FailureCode::InvalidRequest
        );

        let mut timeout = base.clone();
        timeout.arguments["timeout_ms"] = json!(MAX_PROCESS_TIMEOUT_MS + 1);
        assert_eq!(
            engine(&root).authorize(&timeout).unwrap_err().code,
            FailureCode::InvalidRequest
        );

        let mut stdout = base.clone();
        stdout.arguments["stdout_bytes"] = json!(MAX_PROCESS_STDOUT_BYTES + 1);
        assert_eq!(
            engine(&root).authorize(&stdout).unwrap_err().code,
            FailureCode::InvalidRequest
        );

        let mut stderr = base.clone();
        stderr.arguments["stderr_bytes"] = json!(MAX_PROCESS_STDERR_BYTES + 1);
        assert_eq!(
            engine(&root).authorize(&stderr).unwrap_err().code,
            FailureCode::InvalidRequest
        );

        let mut command = base.clone();
        command.arguments["argv"] = json!(["x".repeat(MAX_PROCESS_COMMAND_UTF16)]);
        assert_eq!(
            engine(&root).authorize(&command).unwrap_err().code,
            FailureCode::InvalidRequest
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn process_spawn_rejects_unqualified_shell_and_powershell_executables() {
        let root = temp_root();
        let executable = process_fixture_executable();
        let base = process_request(&executable);
        for unqualified in [
            PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_default())
                .join("System32")
                .join("cmd.exe"),
            PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_default())
                .join("System32")
                .join("WindowsPowerShell")
                .join("v1.0")
                .join("powershell.exe"),
        ] {
            if unqualified.as_os_str().is_empty() || !unqualified.exists() {
                continue;
            }
            let request = process_request(&unqualified);
            assert_eq!(
                engine(&root).authorize(&request).unwrap_err().code,
                FailureCode::CapabilityDenied
            );
        }
        let _ = std::fs::remove_dir_all(root);
        let _ = base;
    }
}
