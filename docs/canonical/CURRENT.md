# Cotra Current Canonical Frontier

Status: IMPLEMENTATION
Date: 2026-09-26
Governance snapshot base: 197e6171e71dc3641b95751da848e4f9c1cf41fc
Evidence ledger: `.specgrain/canonical-evidence.json`

`Governance snapshot base` records the canonical parent from which this snapshot was authored. It deliberately does not claim the eventual merge SHA of the commit containing this file.

## SpecGrain state semantics

- `GRAIN`: authorized/active packet whose acceptance is not yet canonically proven.
- `PROVEN`: acceptance and required qualification are proven, but canonical closeout/governance is incomplete.
- `CLOSED`: machine state corresponding to Diffcipline `COMPLETE_CANONICAL`.

The machine-readable evidence for every `CLOSED` grain is recorded in `.specgrain/canonical-evidence.json` and validated by CI.

## Closed canonical grains

SG-000001 through SG-000012 are `CLOSED` and canonical on the snapshot base.

SG-000012 closeout became canonically effective after PR #25 merged normally as `197e6171e71dc3641b95751da848e4f9c1cf41fc` and exact-main post-merge CI `36255726509` completed SUCCESS.

Qualified SG-000012 behavior remains:
- a zero-capability AppContainer child could not read the Cotra-owned LocalAppData protected-state sentinel;
- tested Cotra/API secret-like variables were absent from the contained child environment;
- contained stdin was NUL/EOF rather than the cotra-mcp -> cotrad authority channel;
- AppContainer identity, pre-resume Job membership, descendant-aware lifecycle handling, and final Job quiescence remained verified;
- no Cotra-owned ACL repair was required;
- no trusted workspace or arbitrary user-file ACL was modified;
- SG-000009, SG-000009A, SG-000010, and SG-000011 regressions remained green.

## Canonical public process boundary retained

Windows public `process.spawn` remains positively restricted to the exact SG-000010-qualified `%SystemRoot%\System32\whoami.exe` target.

Still denied or absent:
- generic executable-registry widening;
- additional public `process.spawn` executable targets;
- `cmd.exe` / raw shell authority;
- public `powershell.run`;
- direct PowerShell executable authority through `process.spawn`;
- caller-provided process environment overrides;
- stdin payload injection;
- process network authority;
- PowerShell remoting;
- detached/background public execution;
- public process kill capability;
- Git mutation;
- browser automation;
- Windows UI Automation/input injection;
- elevation;
- approval bypass or persistent approval reuse.

Historical boundary: SG-000004 closed tunnel supervisor/credential isolation only; live ChatGPT Secure MCP Tunnel E2E remains UNPROVEN until exercised on a real Windows runtime with a real tunnel ID/runtime credential.

## Active frontier

SG-000013 — Private bounded PowerShell containment qualification.

The canonical COTRA-P05 plan still requires bounded PowerShell. SG-000010 through SG-000012 established the public argv process path, descendant-aware lifecycle, and protected-state/authority-channel isolation needed before PowerShell can be evaluated safely.

SG-000013 therefore introduces no public PowerShell authority. Its implementation is limited to a provider-private native Windows qualification path using the exact inbox Windows PowerShell executable and a fixed provider-owned script fixture with:
- `-NoLogo`;
- `-NoProfile`;
- `-NonInteractive`;
- trusted-workspace cwd;
- sanitized environment;
- NUL stdin;
- zero AppContainer network capabilities;
- bounded stdout/stderr and timeout;
- verified AppContainer identity;
- pre-resume Job membership;
- verified destructive termination semantics;
- SG-000011 descendant-aware post-exit drain behavior;
- SG-000012 protected-state and authority-channel isolation regressions.

Caller-provided PowerShell script text, public `powershell.run`, PowerShell through `process.spawn`, profile loading, remoting/network, caller environment overrides, stdin payloads, generic executable widening, Git mutation, browser/UI authority, elevation, approval bypass, and persistent approval reuse remain outside this grain.

After SG-000013 implementation is exact-head qualified, merged, post-merge verified, and canonically closed, repository truth must determine whether the next P05 unit may expose a separately approved public bounded `powershell.run` capability or whether additional containment evidence is required first.

Architecture: Cotra is standalone; Kernux is not a dependency.

Evidence rule: never claim PROVEN or CLOSED without required exact-head, platform, merge, post-merge, and governance evidence.

Language rule: all repository technical content is English only.
