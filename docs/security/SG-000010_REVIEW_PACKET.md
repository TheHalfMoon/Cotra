# SG-000010 review packet

Exact implementation base: `9c1e932d93abd46943e91af1118df0dce5fe35de`

Review focus:
- public authority is limited to `process.spawn` / `spawn`;
- MCP input is argv-first and strict;
- Windows executable policy is positive and narrow: SG-000010 permits only the exact qualified `%SystemRoot%\\System32\\whoami.exe` target;
- direct `cmd.exe`, PowerShell, arbitrary interpreters, and generic executable-registry authority remain outside this grain;
- policy rejects shell strings, env injection, stdin payloads, network widening, background controls, cwd escape, invalid executable paths, and unbounded limits;
- daemon constructs the sanitized canonical plan before approval;
- approval digest binds the exact normalized plan and sanitized environment;
- denied/unavailable approval returns before execution;
- execution reuses the existing AppContainer + Job Object path;
- timeout/output-limit results require verified Job quiescence;
- `TerminationUnverified` remains fail-closed;
- PowerShell, Git mutation, browser/UI, elevation, and public kill authority remain absent.

Known authority boundary:
- arbitrary executable widening is NOT part of SG-000010;
- the generic contained executor's post-primary-exit drain path is not considered qualified for arbitrary descendant-producing executables in this grain;
- descendant-aware post-exit drain/termination hardening is required before a successor executable-registry grain can widen public authority.

Required exact-head evidence:
- Rust Windows/Ubuntu CI;
- Node Windows/Ubuntu CI;
- Governance;
- native Windows public daemon/provider fixture;
- native Windows rejection of direct `cmd.exe` and PowerShell executable requests;
- SG-000009/SG-000009A regressions;
- genuine TypeSafe Jev exact diff;
- Alibaba Open Code Review exact range;
- independent security review;
- zero unresolved blocking review threads.
