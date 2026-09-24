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
    fn validates_appcontainer_profile_names() {
        assert!(validate_appcontainer_name("Cotra.Probe.123").is_ok());
        assert!(validate_appcontainer_name("").is_err());
        assert!(validate_appcontainer_name("Cotra/Probe").is_err());
        assert!(validate_appcontainer_name(&"a".repeat(65)).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn windows_appcontainer_profile_lifecycle_is_available() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let name = format!("Cotra.Probe.{}.{}", std::process::id(), suffix);
        let result = probe_appcontainer_profile(&name).expect("AppContainer lifecycle");
        assert!(result.profile_created);
        assert!(result.sid_derived);
        assert!(result.profile_deleted);
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


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppContainerProbe {
    pub profile_created: bool,
    pub sid_derived: bool,
    pub profile_deleted: bool,
}

pub fn validate_appcontainer_name(name: &str) -> Result<(), ExecutionPlanError> {
    if name.is_empty() || name.encode_utf16().count() > 64 {
        return Err(ExecutionPlanError::new(
            "AppContainer name must contain 1..=64 UTF-16 code units",
        ));
    }
    if !name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ' '))
    {
        return Err(ExecutionPlanError::new(
            "AppContainer name contains a character outside the Windows allowed set",
        ));
    }
    Ok(())
}

#[cfg(windows)]
pub fn probe_appcontainer_profile(name: &str) -> Result<AppContainerProbe, ExecutionPlanError> {
    windows_appcontainer::probe(name)
}

#[cfg(not(windows))]
pub fn probe_appcontainer_profile(_name: &str) -> Result<AppContainerProbe, ExecutionPlanError> {
    Err(ExecutionPlanError::new(
        "AppContainer probing is available only on Windows",
    ))
}

#[cfg(windows)]
mod windows_appcontainer {
    use super::{validate_appcontainer_name, AppContainerProbe, ExecutionPlanError};
    use core::ffi::c_void;
    use std::ptr;

    type Hresult = i32;
    type Psid = *mut c_void;

    #[repr(C)]
    struct SidAndAttributes {
        sid: Psid,
        attributes: u32,
    }

    #[link(name = "userenv")]
    unsafe extern "system" {
        fn CreateAppContainerProfile(
            app_container_name: *const u16,
            display_name: *const u16,
            description: *const u16,
            capabilities: *const SidAndAttributes,
            capability_count: u32,
            app_container_sid: *mut Psid,
        ) -> Hresult;

        fn DeriveAppContainerSidFromAppContainerName(
            app_container_name: *const u16,
            app_container_sid: *mut Psid,
        ) -> Hresult;

        fn DeleteAppContainerProfile(app_container_name: *const u16) -> Hresult;
    }

    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn FreeSid(sid: Psid) -> *mut c_void;
    }

    pub(super) fn probe(name: &str) -> Result<AppContainerProbe, ExecutionPlanError> {
        validate_appcontainer_name(name)?;
        let name = wide(name);
        let display = wide("Cotra execution probe");
        let description = wide("Temporary Cotra AppContainer capability probe");

        let mut created_sid: Psid = ptr::null_mut();
        let create_hr = unsafe {
            CreateAppContainerProfile(
                name.as_ptr(),
                display.as_ptr(),
                description.as_ptr(),
                ptr::null(),
                0,
                &mut created_sid,
            )
        };
        if create_hr != 0 {
            return Err(hresult_error("CreateAppContainerProfile", create_hr));
        }

        if !created_sid.is_null() {
            unsafe {
                FreeSid(created_sid);
            }
        }

        let mut derived_sid: Psid = ptr::null_mut();
        let derive_hr =
            unsafe { DeriveAppContainerSidFromAppContainerName(name.as_ptr(), &mut derived_sid) };
        if derive_hr != 0 {
            let _ = unsafe { DeleteAppContainerProfile(name.as_ptr()) };
            return Err(hresult_error(
                "DeriveAppContainerSidFromAppContainerName",
                derive_hr,
            ));
        }

        if !derived_sid.is_null() {
            unsafe {
                FreeSid(derived_sid);
            }
        }

        let delete_hr = unsafe { DeleteAppContainerProfile(name.as_ptr()) };
        if delete_hr != 0 {
            let _ = unsafe { DeleteAppContainerProfile(name.as_ptr()) };
            return Err(hresult_error("DeleteAppContainerProfile", delete_hr));
        }

        Ok(AppContainerProbe {
            profile_created: true,
            sid_derived: true,
            profile_deleted: true,
        })
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn hresult_error(operation: &str, code: Hresult) -> ExecutionPlanError {
        ExecutionPlanError::new(format!(
            "{operation} failed with HRESULT 0x{:08X}",
            code as u32
        ))
    }
}
