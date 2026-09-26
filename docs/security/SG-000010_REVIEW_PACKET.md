# SG-000010 review packet

Exact implementation base: `9c1e932d93abd46943e91af1118df0dce5fe35de`

Review focus:
- public authority is limited to `process.spawn` / `spawn`;
- MCP input is argv-first and strict;
- policy rejects shell strings, env injection, stdin payloads, network widening, background controls, cwd escape, invalid executable paths, and unbounded limits;
- daemon constructs the sanitized canonical plan before approval;
- approval digest binds the exact normalized plan and sanitized environment;
- denied/unavailable approval returns before execution;
- execution reuses the existing AppContainer + Job Object path;
- timeout/output-limit results require verified Job quiescence;
- `TerminationUnverified` remains fail-closed;
- PowerShell, Git mutation, browser/UI, elevation, and public kill authority remain absent.

Required exact-head evidence:
- Rust Windows/Ubuntu CI;
- Node Windows/Ubuntu CI;
- Governance;
- native Windows public daemon/provider fixture;
- SG-000009/SG-000009A regressions;
- genuine TypeSafe Jev exact diff;
- Alibaba Open Code Review exact range;
- zero unresolved blocking review threads.
