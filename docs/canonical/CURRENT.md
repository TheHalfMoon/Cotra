# Cotra Current Canonical Frontier

Status: IMPLEMENTATION
Date: 2026-09-26
Governance snapshot base: f743ca8923756b7d89f009a9994286a27c80245f
Evidence ledger: `.specgrain/canonical-evidence.json`

`Governance snapshot base` records the canonical parent from which this snapshot was authored. It deliberately does not claim the eventual merge SHA of the commit containing this file.

## SpecGrain state semantics

- `GRAIN`: authorized/active packet whose acceptance is not yet canonically proven.
- `PROVEN`: acceptance and required qualification are proven, but canonical closeout/governance is incomplete.
- `CLOSED`: machine state corresponding to Diffcipline `COMPLETE_CANONICAL`.

The machine-readable evidence for every `CLOSED` grain is recorded in `.specgrain/canonical-evidence.json` and validated by CI.

## Closed canonical grains

SG-000001 through SG-000010 are `CLOSED` and canonical.

SG-000010 closeout evidence:
- qualified implementation head: `248b96c17e3dc387371441ba12b5ee75d41c2495`;
- implementation merge: `388f5de196580d71275a31b84b584f5ac280baec`;
- implementation exact-head CI: `36248970994` — SUCCESS;
- implementation Review Gates: `36248969775` — SUCCESS;
- implementation post-merge CI: `36250111765` — 5/5 SUCCESS;
- closeout exact head: `4f001c1e340b8438dc7b71b9fa8b3be8c50e1bf9`;
- closeout merge: `f743ca8923756b7d89f009a9994286a27c80245f`;
- closeout exact-head CI: `36250486622` — 5/5 SUCCESS;
- closeout Review Gates: `36250486619` — genuine Jev 13/13 hunks, zero findings/blocking findings; Alibaba Open Code Review exact-range delegation SUCCESS;
- closeout post-merge CI: `36250728252` — 5/5 SUCCESS.

Canonical public process authority remains narrow:
- `process.spawn` / `spawn` is argv-first;
- on Windows, the public executable policy remains positively restricted to the exact qualified `%SystemRoot%\System32\whoami.exe` target;
- cwd is workspace-bound;
- caller environment overrides and stdin payloads are absent;
- timeout/stdout/stderr are bounded;
- `network_class=NONE` only;
- every execution requires fresh local approval bound to the normalized plan;
- execution uses zero-capability AppContainer containment plus pre-resume Job membership and verified termination semantics.

Historical boundary: SG-000004 closed tunnel supervisor/credential isolation only; live ChatGPT Secure MCP Tunnel E2E remains UNPROVEN until exercised on a real Windows runtime with a real tunnel ID/runtime credential.

## Active frontier

SG-000011 — Descendant-aware post-exit lifecycle hardening

Branch:
`feat/sg-000011-descendant-lifecycle`

This is the next lawful COTRA-P05 unit because SG-000010 explicitly left the generic contained executor's post-primary-exit pipe-drain behavior unqualified for arbitrary descendant-producing executables, and the canonical security boundary requires this hardening before executable-registry authority can widen.

SG-000011 must prove, on native Windows:
- a deterministic provider-private fixture can exit its primary process while a descendant retains inherited stdout/stderr handles without hanging Cotra;
- post-primary-exit pipe draining is bounded by an explicit internal deadline;
- active descendants are resolved through the existing Job boundary;
- if descendants remain active beyond the bounded drain window, the Job is terminated and zero active Job processes are verified;
- success requires known primary exit, bounded reader terminal state, and Job quiescence;
- inability to prove termination/quiescence returns `TerminationUnverified` and never a stronger success/timeout/output-limit claim;
- race precedence across primary exit, EOF, output overflow, timeout, Job termination, and quiescence is deterministic;
- SG-000009, SG-000009A, and SG-000010 regressions remain green.

## Authority boundary retained during SG-000011

No new public authority is authorized by this grain.

Still denied or absent:
- generic executable-registry widening;
- additional public `process.spawn` executable targets;
- `cmd.exe` / raw shell authority;
- `powershell.run` or direct PowerShell executable authority;
- caller-provided process environment overrides;
- stdin payload injection;
- process network authority;
- detached/background public execution;
- public process kill capability;
- Git mutation;
- browser automation;
- Windows UI Automation/input injection;
- elevation;
- approval bypass or persistent approval reuse.

Only after SG-000011 is `CLOSED` may repository truth determine whether the next P05 unit should widen a positive executable registry, qualify bounded PowerShell, or address another protected-state requirement. No such successor authority is granted here.

Architecture: Cotra is standalone; Kernux is not a dependency.

Evidence rule: never claim PROVEN or CLOSED without required exact-head, platform, merge, post-merge, and governance evidence.

Language rule: all repository technical content is English only.
