# SG-000010 — Approved bounded argv `process.spawn`

Status: IMPLEMENTATION CANDIDATE
Program: COTRA-P05

## Authority delta

SG-000010 adds exactly one public EXECUTE capability: `process.spawn` / `spawn`.

The interface is argv-first. It accepts an absolute executable path and a separate argv array. It does not accept a raw command string, shell text, PowerShell text, caller environment overrides, stdin payloads, background execution, process network capability, public kill authority, elevation, browser/UI automation, or approval reuse.

## Request boundary

The MCP edge and policy kernel independently constrain:
- executable: non-empty absolute local path;
- argv: bounded string array with no NUL bytes;
- cwd: relative path resolving inside the selected trusted workspace;
- timeout: 1 second through 30 minutes;
- stdout: 1 byte through 16 MiB;
- stderr: 1 byte through 4 MiB;
- stdin policy: `null` only;
- network class: `NONE` only.

Unknown process argument fields fail closed. In particular, `command`, `env`, stdin payloads, and detached/background controls are not part of the contract.

## Normalization and approval

`cotrad` builds an `ExecutionPlan` before approval. The plan canonicalizes the executable and cwd, preserves argv order, applies the fixed inherited-environment allowlist, removes Cotra/tunnel and secret-like variables, and validates execution limits.

Every execution requires a fresh independent local approval. The approval digest is SHA-256 over length-prefixed normalized fields with domain separation. It binds:
- workspace identity;
- policy revision;
- canonical executable;
- argv in order;
- canonical cwd;
- timeout/stdout/stderr bounds;
- `stdin=null`;
- `network=NONE`;
- the exact sanitized inherited environment.

Material plan drift therefore changes the approval digest. Denied or unavailable approval returns before the contained executor is called.

## Windows containment

The public path reuses the already-qualified SG-000009/SG-000009A contained executor rather than introducing `std::process::Command` or a shell bypass.

The child is:
1. created suspended in a zero-capability AppContainer;
2. given only NUL stdin plus explicit stdout/stderr pipe handles;
3. assigned to a kill-on-close Job Object;
4. verified as a Job member before resume;
5. verified to have an AppContainer token before resume;
6. resumed only after those checks;
7. monitored with independent timeout/stdout/stderr bounds.

Timeout or output overflow terminates the Job and is promoted to its typed failure only after zero active Job processes are verified. Failure to prove quiescence returns `PROCESS_TERMINATION_UNVERIFIED`.

## Output and evidence

Successful execution returns bounded stdout/stderr, exit code, and containment evidence:
- AppContainer token verified;
- assigned to Job before resume;
- Job quiescent at completion;
- `network_class=NONE`;
- `stdin_policy=null`.

No background process handle or public process identifier is returned.

## Native qualification

The required Windows end-to-end test traverses policy -> approval -> daemon -> process provider using a fixed system inbox `whoami.exe` fixture. It must prove exit/output evidence, AppContainer identity, pre-resume Job membership, and final Job quiescence.

SG-000009 and SG-000009A timeout/output-limit/termination regressions remain mandatory. Exact-head Windows/Ubuntu Rust, Node, Governance, genuine Jev, and Alibaba Open Code Review gates are required before merge.
