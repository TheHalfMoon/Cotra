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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobObjectProbe {
    pub job_created: bool,
    pub kill_on_close_set: bool,
    pub handle_closed: bool,
}

#[cfg(windows)]
pub fn probe_job_object_kill_on_close() -> Result<JobObjectProbe, ExecutionPlanError> {
    windows_job_object::probe()
}

#[cfg(not(windows))]
pub fn probe_job_object_kill_on_close() -> Result<JobObjectProbe, ExecutionPlanError> {
    Err(ExecutionPlanError::new(
        "Job Object probing is available only on Windows",
    ))
}

#[cfg(windows)]
mod windows_job_object {
    use super::{ExecutionPlanError, JobObjectProbe};
    use core::ffi::c_void;
    use std::mem;
    use std::ptr;

    type Handle = *mut c_void;

    const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x0000_2000;
    const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS: i32 = 9;

    #[repr(C)]
    #[derive(Debug, Clone, Copy, Default)]
    struct JobObjectBasicLimitInformation {
        per_process_user_time_limit: i64,
        per_job_user_time_limit: i64,
        limit_flags: u32,
        minimum_working_set_size: usize,
        maximum_working_set_size: usize,
        active_process_limit: u32,
        affinity: usize,
        priority_class: u32,
        scheduling_class: u32,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, Default)]
    struct IoCounters {
        read_operation_count: u64,
        write_operation_count: u64,
        other_operation_count: u64,
        read_transfer_count: u64,
        write_transfer_count: u64,
        other_transfer_count: u64,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, Default)]
    struct JobObjectExtendedLimitInformation {
        basic_limit_information: JobObjectBasicLimitInformation,
        io_info: IoCounters,
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory_used: usize,
        peak_job_memory_used: usize,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateJobObjectW(job_attributes: *const c_void, name: *const u16) -> Handle;
        fn SetInformationJobObject(
            job: Handle,
            information_class: i32,
            information: *const c_void,
            information_length: u32,
        ) -> i32;
        fn CloseHandle(object: Handle) -> i32;
        fn GetLastError() -> u32;
    }

    pub(super) fn probe() -> Result<JobObjectProbe, ExecutionPlanError> {
        let job = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
        if job.is_null() {
            return Err(last_error("CreateJobObjectW"));
        }

        let mut information = JobObjectExtendedLimitInformation::default();
        information.basic_limit_information.limit_flags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

        let configured = unsafe {
            SetInformationJobObject(
                job,
                JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS,
                (&information as *const JobObjectExtendedLimitInformation).cast(),
                mem::size_of::<JobObjectExtendedLimitInformation>() as u32,
            )
        };

        if configured == 0 {
            let error = last_error("SetInformationJobObject");
            unsafe {
                CloseHandle(job);
            }
            return Err(error);
        }

        let closed = unsafe { CloseHandle(job) };
        if closed == 0 {
            return Err(last_error("CloseHandle"));
        }

        Ok(JobObjectProbe {
            job_created: true,
            kill_on_close_set: true,
            handle_closed: true,
        })
    }

    fn last_error(operation: &str) -> ExecutionPlanError {
        let code = unsafe { GetLastError() };
        ExecutionPlanError::new(format!("{operation} failed with Win32 error {code}"))
    }
}

#[cfg(all(test, windows))]
mod job_object_tests {
    use super::*;

    #[test]
    fn windows_job_object_kill_on_close_is_available() {
        let result = probe_job_object_kill_on_close().expect("Job Object lifecycle");
        assert!(result.job_created);
        assert!(result.kill_on_close_set);
        assert!(result.handle_closed);
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainedLaunchProbe {
    pub profile_created: bool,
    pub process_created_suspended: bool,
    pub appcontainer_token_verified: bool,
    pub assigned_to_job_before_resume: bool,
    pub resumed: bool,
    pub child_completed: bool,
    pub exit_code: u32,
    pub profile_deleted: bool,
}

#[cfg(windows)]
pub fn probe_contained_appcontainer_job_launch(
    name: &str,
) -> Result<ContainedLaunchProbe, ExecutionPlanError> {
    windows_contained_launch::probe(name)
}

#[cfg(not(windows))]
pub fn probe_contained_appcontainer_job_launch(
    _name: &str,
) -> Result<ContainedLaunchProbe, ExecutionPlanError> {
    Err(ExecutionPlanError::new(
        "contained AppContainer launch probing is available only on Windows",
    ))
}

#[cfg(windows)]
mod windows_contained_launch {
    use super::{validate_appcontainer_name, ContainedLaunchProbe, ExecutionPlanError};
    use core::ffi::c_void;
    use std::ffi::OsStr;
    use std::mem;
    use std::os::windows::ffi::OsStrExt;
    use std::path::{Path, PathBuf};
    use std::ptr;

    type Handle = *mut c_void;
    type Psid = *mut c_void;
    type Hresult = i32;

    const CREATE_SUSPENDED: u32 = 0x0000_0004;
    const CREATE_UNICODE_ENVIRONMENT: u32 = 0x0000_0400;
    const EXTENDED_STARTUPINFO_PRESENT: u32 = 0x0008_0000;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES: usize = 131_081;
    const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x0000_2000;
    const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS: i32 = 9;
    const TOKEN_QUERY: u32 = 0x0008;
    const TOKEN_IS_APP_CONTAINER: i32 = 29;
    const WAIT_OBJECT_0: u32 = 0;
    const WAIT_TIMEOUT: u32 = 258;
    const WAIT_FAILED: u32 = u32::MAX;
    const RESUME_FAILED: u32 = u32::MAX;
    const PROBE_TIMEOUT_MS: u32 = 15_000;

    #[repr(C)]
    struct SidAndAttributes {
        sid: Psid,
        attributes: u32,
    }

    #[repr(C)]
    struct SecurityCapabilities {
        app_container_sid: Psid,
        capabilities: *mut SidAndAttributes,
        capability_count: u32,
        reserved: u32,
    }

    #[repr(C)]
    struct StartupInfoW {
        cb: u32,
        lp_reserved: *mut u16,
        lp_desktop: *mut u16,
        lp_title: *mut u16,
        dw_x: u32,
        dw_y: u32,
        dw_x_size: u32,
        dw_y_size: u32,
        dw_x_count_chars: u32,
        dw_y_count_chars: u32,
        dw_fill_attribute: u32,
        dw_flags: u32,
        w_show_window: u16,
        cb_reserved_2: u16,
        lp_reserved_2: *mut u8,
        h_std_input: Handle,
        h_std_output: Handle,
        h_std_error: Handle,
    }

    #[repr(C)]
    struct StartupInfoExW {
        startup_info: StartupInfoW,
        attribute_list: *mut c_void,
    }

    #[repr(C)]
    struct ProcessInformation {
        process: Handle,
        thread: Handle,
        process_id: u32,
        thread_id: u32,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, Default)]
    struct JobObjectBasicLimitInformation {
        per_process_user_time_limit: i64,
        per_job_user_time_limit: i64,
        limit_flags: u32,
        minimum_working_set_size: usize,
        maximum_working_set_size: usize,
        active_process_limit: u32,
        affinity: usize,
        priority_class: u32,
        scheduling_class: u32,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, Default)]
    struct IoCounters {
        read_operation_count: u64,
        write_operation_count: u64,
        other_operation_count: u64,
        read_transfer_count: u64,
        write_transfer_count: u64,
        other_transfer_count: u64,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, Default)]
    struct JobObjectExtendedLimitInformation {
        basic_limit_information: JobObjectBasicLimitInformation,
        io_info: IoCounters,
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory_used: usize,
        peak_job_memory_used: usize,
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
        fn DeleteAppContainerProfile(app_container_name: *const u16) -> Hresult;
    }

    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn FreeSid(sid: Psid) -> *mut c_void;
        fn OpenProcessToken(
            process_handle: Handle,
            desired_access: u32,
            token_handle: *mut Handle,
        ) -> i32;
        fn GetTokenInformation(
            token_handle: Handle,
            token_information_class: i32,
            token_information: *mut c_void,
            token_information_length: u32,
            return_length: *mut u32,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn InitializeProcThreadAttributeList(
            attribute_list: *mut c_void,
            attribute_count: u32,
            flags: u32,
            size: *mut usize,
        ) -> i32;
        fn UpdateProcThreadAttribute(
            attribute_list: *mut c_void,
            flags: u32,
            attribute: usize,
            value: *mut c_void,
            size: usize,
            previous_value: *mut c_void,
            return_size: *mut usize,
        ) -> i32;
        fn DeleteProcThreadAttributeList(attribute_list: *mut c_void);
        fn GetProcessHeap() -> Handle;
        fn HeapAlloc(heap: Handle, flags: u32, bytes: usize) -> *mut c_void;
        fn HeapFree(heap: Handle, flags: u32, memory: *mut c_void) -> i32;
        fn CreateProcessW(
            application_name: *const u16,
            command_line: *mut u16,
            process_attributes: *const c_void,
            thread_attributes: *const c_void,
            inherit_handles: i32,
            creation_flags: u32,
            environment: *mut c_void,
            current_directory: *const u16,
            startup_info: *const StartupInfoW,
            process_information: *mut ProcessInformation,
        ) -> i32;
        fn CreateJobObjectW(job_attributes: *const c_void, name: *const u16) -> Handle;
        fn SetInformationJobObject(
            job: Handle,
            information_class: i32,
            information: *const c_void,
            information_length: u32,
        ) -> i32;
        fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
        fn IsProcessInJob(process: Handle, job: Handle, result: *mut i32) -> i32;
        fn ResumeThread(thread: Handle) -> u32;
        fn WaitForSingleObject(handle: Handle, milliseconds: u32) -> u32;
        fn GetExitCodeProcess(process: Handle, exit_code: *mut u32) -> i32;
        fn TerminateProcess(process: Handle, exit_code: u32) -> i32;
        fn CloseHandle(object: Handle) -> i32;
        fn GetLastError() -> u32;
    }

    struct OwnedHandle(Handle);

    impl OwnedHandle {
        fn new(handle: Handle, operation: &str) -> Result<Self, ExecutionPlanError> {
            if handle.is_null() {
                Err(last_error(operation))
            } else {
                Ok(Self(handle))
            }
        }

        fn raw(&self) -> Handle {
            self.0
        }
    }

    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    CloseHandle(self.0);
                }
            }
        }
    }

    struct AppContainerProfile {
        name: Vec<u16>,
        sid: Psid,
        deleted: bool,
    }

    impl AppContainerProfile {
        fn create(name: &str) -> Result<Self, ExecutionPlanError> {
            validate_appcontainer_name(name)?;
            let name_wide = wide(OsStr::new(name));
            let display = wide(OsStr::new("Cotra contained launch probe"));
            let description = wide(OsStr::new(
                "Temporary zero-capability Cotra AppContainer launch probe",
            ));
            let mut sid: Psid = ptr::null_mut();

            let result = unsafe {
                CreateAppContainerProfile(
                    name_wide.as_ptr(),
                    display.as_ptr(),
                    description.as_ptr(),
                    ptr::null(),
                    0,
                    &mut sid,
                )
            };
            if result != 0 {
                return Err(hresult_error("CreateAppContainerProfile", result));
            }
            if sid.is_null() {
                let _ = unsafe { DeleteAppContainerProfile(name_wide.as_ptr()) };
                return Err(ExecutionPlanError::new(
                    "CreateAppContainerProfile returned a null SID",
                ));
            }

            Ok(Self {
                name: name_wide,
                sid,
                deleted: false,
            })
        }

        fn sid(&self) -> Psid {
            self.sid
        }

        fn delete(&mut self) -> Result<(), ExecutionPlanError> {
            if self.deleted {
                return Ok(());
            }
            let result = unsafe { DeleteAppContainerProfile(self.name.as_ptr()) };
            if result != 0 {
                return Err(hresult_error("DeleteAppContainerProfile", result));
            }
            self.deleted = true;
            Ok(())
        }
    }

    impl Drop for AppContainerProfile {
        fn drop(&mut self) {
            if !self.deleted {
                unsafe {
                    DeleteAppContainerProfile(self.name.as_ptr());
                }
            }
            if !self.sid.is_null() {
                unsafe {
                    FreeSid(self.sid);
                }
            }
        }
    }

    struct AttributeList {
        heap: Handle,
        list: *mut c_void,
    }

    impl AttributeList {
        fn with_security_capabilities(
            security: &mut SecurityCapabilities,
        ) -> Result<Self, ExecutionPlanError> {
            let heap = unsafe { GetProcessHeap() };
            if heap.is_null() {
                return Err(last_error("GetProcessHeap"));
            }

            let mut bytes = 0usize;
            unsafe {
                InitializeProcThreadAttributeList(ptr::null_mut(), 1, 0, &mut bytes);
            }
            if bytes == 0 {
                return Err(last_error("InitializeProcThreadAttributeList(size)"));
            }

            let list = unsafe { HeapAlloc(heap, 0, bytes) };
            if list.is_null() {
                return Err(last_error("HeapAlloc(attribute list)"));
            }

            let initialized = unsafe { InitializeProcThreadAttributeList(list, 1, 0, &mut bytes) };
            if initialized == 0 {
                let error = last_error("InitializeProcThreadAttributeList");
                unsafe {
                    HeapFree(heap, 0, list);
                }
                return Err(error);
            }

            let updated = unsafe {
                UpdateProcThreadAttribute(
                    list,
                    0,
                    PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES,
                    (security as *mut SecurityCapabilities).cast(),
                    mem::size_of::<SecurityCapabilities>(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                )
            };
            if updated == 0 {
                let error = last_error("UpdateProcThreadAttribute(SECURITY_CAPABILITIES)");
                unsafe {
                    DeleteProcThreadAttributeList(list);
                    HeapFree(heap, 0, list);
                }
                return Err(error);
            }

            Ok(Self { heap, list })
        }

        fn raw(&self) -> *mut c_void {
            self.list
        }
    }

    impl Drop for AttributeList {
        fn drop(&mut self) {
            if !self.list.is_null() {
                unsafe {
                    DeleteProcThreadAttributeList(self.list);
                    HeapFree(self.heap, 0, self.list);
                }
            }
        }
    }

    struct JobObject {
        handle: OwnedHandle,
    }

    impl JobObject {
        fn kill_on_close() -> Result<Self, ExecutionPlanError> {
            let handle = OwnedHandle::new(
                unsafe { CreateJobObjectW(ptr::null(), ptr::null()) },
                "CreateJobObjectW",
            )?;

            let mut information = JobObjectExtendedLimitInformation::default();
            information.basic_limit_information.limit_flags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            let configured = unsafe {
                SetInformationJobObject(
                    handle.raw(),
                    JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS,
                    (&information as *const JobObjectExtendedLimitInformation).cast(),
                    mem::size_of::<JobObjectExtendedLimitInformation>() as u32,
                )
            };
            if configured == 0 {
                return Err(last_error("SetInformationJobObject"));
            }

            Ok(Self { handle })
        }

        fn assign_before_resume(&self, child: &ChildProcess) -> Result<(), ExecutionPlanError> {
            let assigned =
                unsafe { AssignProcessToJobObject(self.handle.raw(), child.process.raw()) };
            if assigned == 0 {
                return Err(last_error("AssignProcessToJobObject"));
            }

            let mut in_job = 0i32;
            let checked =
                unsafe { IsProcessInJob(child.process.raw(), self.handle.raw(), &mut in_job) };
            if checked == 0 {
                return Err(last_error("IsProcessInJob"));
            }
            if in_job == 0 {
                return Err(ExecutionPlanError::new(
                    "child process was not associated with the Cotra Job Object",
                ));
            }
            Ok(())
        }
    }

    struct ChildProcess {
        process: OwnedHandle,
        thread: OwnedHandle,
        completed: bool,
    }

    impl ChildProcess {
        fn create_suspended(profile_sid: Psid) -> Result<Self, ExecutionPlanError> {
            let system_root = std::env::var_os("SystemRoot")
                .ok_or_else(|| ExecutionPlanError::new("SystemRoot is unavailable"))?;
            let system32 = PathBuf::from(&system_root).join("System32");
            let executable = system32.join("cmd.exe");
            if !executable.is_file() {
                return Err(ExecutionPlanError::new(format!(
                    "contained probe executable does not exist: {}",
                    executable.display()
                )));
            }

            let mut security = SecurityCapabilities {
                app_container_sid: profile_sid,
                capabilities: ptr::null_mut(),
                capability_count: 0,
                reserved: 0,
            };
            let attributes = AttributeList::with_security_capabilities(&mut security)?;

            let mut startup: StartupInfoExW = unsafe { mem::zeroed() };
            startup.startup_info.cb = mem::size_of::<StartupInfoExW>() as u32;
            startup.attribute_list = attributes.raw();

            let application = wide(executable.as_os_str());
            let mut command_line = command_line_for_cmd(&executable);
            let current_directory = wide(system32.as_os_str());
            let mut environment = minimal_environment_block(&system_root);

            let mut process_information: ProcessInformation = unsafe { mem::zeroed() };
            let created = unsafe {
                CreateProcessW(
                    application.as_ptr(),
                    command_line.as_mut_ptr(),
                    ptr::null(),
                    ptr::null(),
                    0,
                    CREATE_SUSPENDED
                        | CREATE_UNICODE_ENVIRONMENT
                        | EXTENDED_STARTUPINFO_PRESENT
                        | CREATE_NO_WINDOW,
                    environment.as_mut_ptr().cast(),
                    current_directory.as_ptr(),
                    (&startup as *const StartupInfoExW).cast(),
                    &mut process_information,
                )
            };
            if created == 0 {
                return Err(last_error("CreateProcessW(AppContainer suspended)"));
            }

            let process = OwnedHandle::new(process_information.process, "CreateProcessW process")?;
            let thread = match OwnedHandle::new(process_information.thread, "CreateProcessW thread")
            {
                Ok(thread) => thread,
                Err(error) => {
                    unsafe {
                        TerminateProcess(process.raw(), 1);
                        WaitForSingleObject(process.raw(), 5_000);
                    }
                    return Err(error);
                }
            };

            Ok(Self {
                process,
                thread,
                completed: false,
            })
        }

        fn verify_appcontainer_token(&self) -> Result<(), ExecutionPlanError> {
            let mut raw_token: Handle = ptr::null_mut();
            let opened =
                unsafe { OpenProcessToken(self.process.raw(), TOKEN_QUERY, &mut raw_token) };
            if opened == 0 {
                return Err(last_error("OpenProcessToken"));
            }
            let token = OwnedHandle::new(raw_token, "OpenProcessToken handle")?;

            let mut is_appcontainer = 0u32;
            let mut returned = 0u32;
            let queried = unsafe {
                GetTokenInformation(
                    token.raw(),
                    TOKEN_IS_APP_CONTAINER,
                    (&mut is_appcontainer as *mut u32).cast(),
                    mem::size_of::<u32>() as u32,
                    &mut returned,
                )
            };
            if queried == 0 {
                return Err(last_error("GetTokenInformation(TokenIsAppContainer)"));
            }
            if is_appcontainer == 0 {
                return Err(ExecutionPlanError::new(
                    "contained child token is not marked as AppContainer",
                ));
            }
            Ok(())
        }

        fn resume(&self) -> Result<(), ExecutionPlanError> {
            let previous = unsafe { ResumeThread(self.thread.raw()) };
            if previous == RESUME_FAILED {
                return Err(last_error("ResumeThread"));
            }
            Ok(())
        }

        fn wait_for_completion(&mut self) -> Result<u32, ExecutionPlanError> {
            let wait = unsafe { WaitForSingleObject(self.process.raw(), PROBE_TIMEOUT_MS) };
            match wait {
                WAIT_OBJECT_0 => {}
                WAIT_TIMEOUT => {
                    self.terminate_best_effort();
                    return Err(ExecutionPlanError::new(
                        "contained child did not exit before the probe timeout",
                    ));
                }
                WAIT_FAILED => {
                    let error = last_error("WaitForSingleObject");
                    self.terminate_best_effort();
                    return Err(error);
                }
                other => {
                    self.terminate_best_effort();
                    return Err(ExecutionPlanError::new(format!(
                        "WaitForSingleObject returned unexpected status {other}"
                    )));
                }
            }

            let mut exit_code = 0u32;
            let read = unsafe { GetExitCodeProcess(self.process.raw(), &mut exit_code) };
            if read == 0 {
                return Err(last_error("GetExitCodeProcess"));
            }
            self.completed = true;
            Ok(exit_code)
        }

        fn terminate_best_effort(&mut self) {
            unsafe {
                TerminateProcess(self.process.raw(), 1);
                WaitForSingleObject(self.process.raw(), 5_000);
            }
            self.completed = true;
        }
    }

    impl Drop for ChildProcess {
        fn drop(&mut self) {
            if !self.completed {
                self.terminate_best_effort();
            }
        }
    }

    pub(super) fn probe(name: &str) -> Result<ContainedLaunchProbe, ExecutionPlanError> {
        let mut profile = AppContainerProfile::create(name)?;
        let job = JobObject::kill_on_close()?;
        let mut child = ChildProcess::create_suspended(profile.sid())?;

        if let Err(error) = job.assign_before_resume(&child) {
            child.terminate_best_effort();
            return Err(error);
        }

        child.verify_appcontainer_token()?;
        child.resume()?;
        let exit_code = child.wait_for_completion()?;
        profile.delete()?;

        Ok(ContainedLaunchProbe {
            profile_created: true,
            process_created_suspended: true,
            appcontainer_token_verified: true,
            assigned_to_job_before_resume: true,
            resumed: true,
            child_completed: true,
            exit_code,
            profile_deleted: true,
        })
    }

    fn command_line_for_cmd(executable: &Path) -> Vec<u16> {
        let mut command = Vec::new();
        command.push('"' as u16);
        command.extend(executable.as_os_str().encode_wide());
        command.push('"' as u16);
        command.extend(" /d /q /c exit 0".encode_utf16());
        command.push(0);
        command
    }

    fn minimal_environment_block(system_root: &OsStr) -> Vec<u16> {
        let mut environment = Vec::new();
        push_environment_entry(&mut environment, "SystemRoot", system_root);
        push_environment_entry(&mut environment, "WINDIR", system_root);
        environment.push(0);
        environment
    }

    fn push_environment_entry(buffer: &mut Vec<u16>, key: &str, value: &OsStr) {
        buffer.extend(key.encode_utf16());
        buffer.push('=' as u16);
        buffer.extend(value.encode_wide());
        buffer.push(0);
    }

    fn wide(value: &OsStr) -> Vec<u16> {
        value.encode_wide().chain(std::iter::once(0)).collect()
    }

    fn last_error(operation: &str) -> ExecutionPlanError {
        let code = unsafe { GetLastError() };
        ExecutionPlanError::new(format!("{operation} failed with Win32 error {code}"))
    }

    fn hresult_error(operation: &str, code: Hresult) -> ExecutionPlanError {
        ExecutionPlanError::new(format!(
            "{operation} failed with HRESULT 0x{:08X}",
            code as u32
        ))
    }
}

#[cfg(all(test, windows))]
mod contained_launch_tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn windows_appcontainer_child_is_job_assigned_before_resume() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let name = format!(
            "Cotra.Contained.{}.{}",
            std::process::id(),
            suffix
        );

        let result =
            probe_contained_appcontainer_job_launch(&name).expect("contained child launch");
        assert!(result.profile_created);
        assert!(result.process_created_suspended);
        assert!(result.appcontainer_token_verified);
        assert!(result.assigned_to_job_before_resume);
        assert!(result.resumed);
        assert!(result.child_completed);
        assert_eq!(result.exit_code, 0);
        assert!(result.profile_deleted);
    }
}
