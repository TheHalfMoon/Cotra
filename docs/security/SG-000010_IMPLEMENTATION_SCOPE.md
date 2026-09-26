# SG-000010 implementation scope

Base: `9c1e932d93abd46943e91af1118df0dce5fe35de`
Grain: `SG-000010`
Program: `COTRA-P05`

This implementation opens only the approved argv-first `process.spawn` contract described by SG-000010.

Allowed authority delta:
- one public `process.spawn` / `spawn` operation;
- absolute executable + argv array;
- workspace-bound cwd;
- fixed sanitized inherited environment;
- null stdin;
- bounded timeout/stdout/stderr;
- `network_class=NONE`;
- fresh exact-plan local approval;
- zero-capability AppContainer and verified Job containment.

Explicitly absent:
- raw command strings;
- `cmd.exe`/shell as an API primitive;
- `powershell.run`;
- caller environment overrides;
- stdin payloads;
- process network capability;
- detached/background execution;
- public process kill;
- Git mutation;
- browser/UI authority;
- elevation;
- approval bypass or reuse.

The existing SG-000009/SG-000009A contained executor remains the security boundary. This grain does not introduce an alternate process-launch path.
