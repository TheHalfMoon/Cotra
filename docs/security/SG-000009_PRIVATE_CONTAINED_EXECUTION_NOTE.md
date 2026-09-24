# SG-000009 private contained argv execution note

Date: 2026-09-24

SG-000009 qualifies a provider-internal Windows executor. It does not register a process capability and is not reachable from policy, cotrad, or MCP.

The qualification fixture is deliberately fixed:

- executable: the operating-system `whoami.exe` resolved relative to `GetSystemDirectoryW`;
- argv: no arguments; `/all` is excluded because its claims query is unavailable in the zero-capability AppContainer and returns exit code 1;
- cwd: an explicit temporary qualification root;
- environment: the existing fixed-name, case-insensitive allowlist with Cotra/tunnel/secret values excluded;
- stdin: `NUL` with no caller input;
- stdout/stderr: separate bounded pipes;
- process: zero-capability AppContainer, created suspended;
- lifecycle: Job Object assignment and `IsProcessInJob` verification before resume;
- completion: finite wait and exit-code evidence;
- failure: Job termination followed by active-process quiescence verification.

The child receives only three explicitly listed inheritable handles through `PROC_THREAD_ATTRIBUTE_HANDLE_LIST`: NUL stdin, the stdout pipe writer, and the stderr pipe writer. Other process handles are not authorized for inheritance. Parent pipe writers are closed immediately after process creation. The implementation contains typed output-limit and timeout termination branches, but SG-000009 does not claim runtime qualification of those destructive paths. Their descendant-tree fixtures and runtime evidence are successor work.

This grain does not claim arbitrary process execution, workspace authority, network access, PowerShell, browser control, UI input, approval, or elevation. The existing policy allowlist continues to deny `process.spawn`.

The native Windows qualification test proves the fixed no-argument fixture, workspace-bound cwd binding, bounded output capture, AppContainer token, pre-resume Job membership, exit code, and Job quiescence. A prior native diagnostic proved that `whoami.exe /all` reports `Unable to get user claims information.` and exits 1 in this zero-capability AppContainer; `/all` is therefore not the success fixture. Timeout/output-overflow and descendant-tree fixtures remain successor work and are not claimed here.

No donor implementation source was copied. The design uses Microsoft AppContainer, Job Object, pipe, and process APIs documented by Win32.
