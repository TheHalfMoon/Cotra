use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const SECRETISH: &[&str] = &[
    "SECRET",
    "TOKEN",
    "PASSWORD",
    "PASSWD",
    "CREDENTIAL",
    "API_KEY",
    "PRIVATE_KEY",
    "TUNNEL_KEY",
];

const FORBIDDEN_COTRA_ENV: &[&str] = &[
    "COTRA_TUNNEL_ID",
    "COTRA_TUNNEL_CLIENT",
    "COTRA_TUNNEL_KEY_FILE",
    "COTRA_TUNNEL_HEALTH_URL_FILE",
    "COTRA_DAEMON",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionLimits {
    pub timeout: Duration,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            stdout_bytes: 2 * 1024 * 1024,
            stderr_bytes: 256 * 1024,
        }
    }
}

impl ExecutionLimits {
    pub fn validate(&self) -> Result<(), ExecutionPlanError> {
        if self.timeout.is_zero() || self.timeout > Duration::from_secs(30 * 60) {
            return Err(ExecutionPlanError::new(
                "timeout must be between 1 second and 30 minutes",
            ));
        }
        if self.stdout_bytes == 0 || self.stdout_bytes > 16 * 1024 * 1024 {
            return Err(ExecutionPlanError::new(
                "stdout limit must be between 1 byte and 16 MiB",
            ));
        }
        if self.stderr_bytes == 0 || self.stderr_bytes > 4 * 1024 * 1024 {
            return Err(ExecutionPlanError::new(
                "stderr limit must be between 1 byte and 4 MiB",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub executable: PathBuf,
    pub argv: Vec<String>,
    pub cwd: PathBuf,
    pub env: BTreeMap<String, String>,
    pub limits: ExecutionLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlanError {
    pub message: String,
}

impl ExecutionPlanError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub fn build_execution_plan(
    workspace_root: impl AsRef<Path>,
    executable: impl AsRef<Path>,
    argv: &[String],
    requested_cwd: impl AsRef<Path>,
    source_env: &BTreeMap<String, String>,
    limits: ExecutionLimits,
) -> Result<ExecutionPlan, ExecutionPlanError> {
    limits.validate()?;

    let workspace_root = canonical_dir(workspace_root.as_ref(), "workspace root")?;
    let executable = executable.as_ref();
    if !executable.is_absolute() {
        return Err(ExecutionPlanError::new(
            "execution executable must be an absolute path",
        ));
    }
    let executable = canonical_file(executable, "execution executable")?;

    for arg in argv {
        if arg.contains('\0') {
            return Err(ExecutionPlanError::new("argv contains a NUL byte"));
        }
    }

    let requested_cwd = requested_cwd.as_ref();
    let cwd = if requested_cwd.as_os_str().is_empty() || requested_cwd == Path::new(".") {
        workspace_root.clone()
    } else if requested_cwd.is_absolute() {
        canonical_dir(requested_cwd, "execution cwd")?
    } else {
        canonical_dir(&workspace_root.join(requested_cwd), "execution cwd")?
    };
    if cwd != workspace_root && !cwd.starts_with(&workspace_root) {
        return Err(ExecutionPlanError::new(
            "execution cwd resolves outside the trusted workspace",
        ));
    }

    Ok(ExecutionPlan {
        executable,
        argv: argv.to_vec(),
        cwd,
        env: sanitized_execution_env(source_env),
        limits,
    })
}

pub fn sanitized_execution_env(source: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (name, value) in source {
        let upper = name.to_ascii_uppercase();
        if SECRETISH.iter().any(|needle| upper.contains(needle)) {
            continue;
        }
        if FORBIDDEN_COTRA_ENV
            .iter()
            .any(|forbidden| upper == *forbidden)
        {
            continue;
        }
        if allowed_execution_env(name) {
            out.insert(name.clone(), value.clone());
        }
    }
    out
}

fn allowed_execution_env(name: &str) -> bool {
    matches!(
        name,
        "PATH"
            | "Path"
            | "PATHEXT"
            | "SystemRoot"
            | "WINDIR"
            | "TEMP"
            | "TMP"
            | "HOME"
            | "USERPROFILE"
            | "LOCALAPPDATA"
            | "APPDATA"
            | "PROGRAMDATA"
            | "ProgramFiles"
            | "ProgramFiles(x86)"
            | "LANG"
            | "LC_ALL"
    )
}

fn canonical_dir(path: &Path, label: &str) -> Result<PathBuf, ExecutionPlanError> {
    let resolved = fs::canonicalize(path)
        .map_err(|error| ExecutionPlanError::new(format!("resolve {label}: {error}")))?;
    if !resolved.is_dir() {
        return Err(ExecutionPlanError::new(format!(
            "{label} is not a directory"
        )));
    }
    Ok(resolved)
}

fn canonical_file(path: &Path, label: &str) -> Result<PathBuf, ExecutionPlanError> {
    let resolved = fs::canonicalize(path)
        .map_err(|error| ExecutionPlanError::new(format!("resolve {label}: {error}")))?;
    if !resolved.is_file() {
        return Err(ExecutionPlanError::new(format!("{label} is not a file")));
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn root(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("cotra-process-{name}-{suffix}"));
        fs::create_dir_all(&root).expect("create root");
        root
    }

    fn executable(root: &Path) -> PathBuf {
        let path = root.join("tool.bin");
        fs::write(&path, b"test").expect("seed executable");
        path
    }

    #[test]
    fn creates_bounded_plan_inside_workspace() {
        let workspace = root("ok");
        let work = workspace.join("project");
        fs::create_dir_all(&work).unwrap();
        let exe = executable(&workspace);
        let env = BTreeMap::from([
            ("PATH".to_owned(), "bin".to_owned()),
            ("OPENAI_API_KEY".to_owned(), "secret".to_owned()),
            ("COTRA_TUNNEL_KEY_FILE".to_owned(), "secret-path".to_owned()),
        ]);

        let plan = build_execution_plan(
            &workspace,
            &exe,
            &["--version".into()],
            "project",
            &env,
            ExecutionLimits::default(),
        )
        .unwrap();

        assert_eq!(plan.cwd, fs::canonicalize(work).unwrap());
        assert_eq!(plan.env.get("PATH").map(String::as_str), Some("bin"));
        assert!(!plan.env.contains_key("OPENAI_API_KEY"));
        assert!(!plan.env.contains_key("COTRA_TUNNEL_KEY_FILE"));
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn rejects_relative_executable() {
        let workspace = root("relative-exe");
        let error = build_execution_plan(
            &workspace,
            "tool.exe",
            &[],
            ".",
            &BTreeMap::new(),
            ExecutionLimits::default(),
        )
        .unwrap_err();
        assert!(error.message.contains("absolute path"));
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn rejects_cwd_escape() {
        let workspace = root("workspace");
        let outside = root("outside");
        let exe = executable(&workspace);
        let error = build_execution_plan(
            &workspace,
            &exe,
            &[],
            &outside,
            &BTreeMap::new(),
            ExecutionLimits::default(),
        )
        .unwrap_err();
        assert!(error.message.contains("outside the trusted workspace"));
        let _ = fs::remove_dir_all(workspace);
        let _ = fs::remove_dir_all(outside);
    }

    #[test]
    fn rejects_nul_argv() {
        let workspace = root("nul");
        let exe = executable(&workspace);
        let error = build_execution_plan(
            &workspace,
            &exe,
            &["bad\0arg".into()],
            ".",
            &BTreeMap::new(),
            ExecutionLimits::default(),
        )
        .unwrap_err();
        assert!(error.message.contains("NUL"));
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn rejects_unbounded_limits() {
        let workspace = root("limits");
        let exe = executable(&workspace);
        let error = build_execution_plan(
            &workspace,
            &exe,
            &[],
            ".",
            &BTreeMap::new(),
            ExecutionLimits {
                timeout: Duration::from_secs(31 * 60),
                stdout_bytes: 1,
                stderr_bytes: 1,
            },
        )
        .unwrap_err();
        assert!(error.message.contains("30 minutes"));
        let _ = fs::remove_dir_all(workspace);
    }
}
