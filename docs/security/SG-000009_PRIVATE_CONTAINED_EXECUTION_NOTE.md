# SG-000009 private contained argv execution note

Date: 2026-09-24

SG-000009 qualifies a provider-internal Windows executor. It does not register a process capability and is not reachable from policy, cotrad, or MCP.

The qualification fixture is deliberately fixed:

- executable: the operating-system `whoami.exe` resolved relative to `GetSystemDirectoryW`;
- argv: exactly `/all`;
- cwd: an explicit temporary qualification root;
- environment: the existing fixed-name, case-insensitive allowlist with Cotra/tunnel/secret values excluded;
- stdin: `NUL` with no caller input;
- stdout/stderr: separate bounded pipes;
- process: zero-capability AppContainer, created suspended;
- lifecycle: Job Object assignment and `IsProcessInJob` verification before resume;
- completion: finite wait and exit-code evidence;
- failure: Job termination followed by active-process quiescence verification.

The child receives only three explicitly listed inheritable handles through `PROC_THREAD_ATTRIBUTE_HANDLE_LIST`. Other process handles are not inherited. Parent pipe writers are closed immediately after process creation. Output-limit and timeout paths terminate the Job and return typed `OutputLimit` or `ProcessTimeout` only after the Job reports zero active processes; otherwise they return `TerminationUnverified`.

This grain does not claim arbitrary process execution, workspace authority, network access, PowerShell, browser control, UI input, approval, or elevation. The existing policy allowlist continues to deny `process.spawn`.

The native Windows qualification test proves the fixed fixture, bounded output, AppContainer token, pre-resume Job membership, exit code, and Job quiescence. Timeout/output-overflow descendant fixtures remain successor work and are not claimed here.

No donor implementation source was copied. The design uses Microsoft AppContainer, Job Object, pipe, and process APIs documented by Win32.
