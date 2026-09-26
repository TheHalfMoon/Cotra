# Cotra Current Canonical Frontier

Status: IMPLEMENTATION
Date: 2026-09-26
Governance snapshot base: be1e6a28c77d25a8f9f1848c02df290ac74da734
Evidence ledger: `.specgrain/canonical-evidence.json`

`Governance snapshot base` records the canonical parent from which this snapshot was authored. It deliberately does not claim the eventual merge SHA of the commit containing this file.

## SpecGrain state semantics

- `GRAIN`: authorized/active packet whose acceptance is not yet canonically proven.
- `PROVEN`: acceptance and required qualification are proven, but canonical closeout/governance is incomplete.
- `CLOSED`: machine state corresponding to Diffcipline `COMPLETE_CANONICAL`.

The machine-readable evidence for every `CLOSED` grain is recorded in `.specgrain/canonical-evidence.json` and validated by CI.

## Closed canonical grains

SG-000001 through SG-000009A are `CLOSED` and canonical after governance reconciliation merge `be1e6a28c77d25a8f9f1848c02df290ac74da734` and post-merge CI `36239819409` (5/5 SUCCESS).

Important historical boundaries remain explicit:
- SG-000004 closed the tunnel supervisor/credential-isolation grain; live ChatGPT Secure MCP Tunnel E2E remains UNPROVEN until exercised on a real Windows runtime with a real tunnel ID/runtime credential.
- SG-000009 closed the fixed provider-private no-argument `whoami.exe` AppContainer success path.
- SG-000009A closed provider-private destructive timeout/stdout-limit/stderr-limit termination qualification with verified zero active Job processes and fail-closed `TerminationUnverified` semantics.

## Canonical capabilities through the snapshot base

- `system.status`
- `workspace.get`
- `fs.stat/list/read/search`
- approved `fs.write` preview/write
- `git.status/diff/log`
- secure tunnel-client supervisor with file-referenced runtime credential isolation
- internal protected execution planning contract
- native Windows AppContainer profile primitive
- native Windows Job Object kill-on-close primitive
- native Windows contained fixed-child launch qualification
- native Windows destructive timeout/output-limit termination with verified Job quiescence, provider-private only

## Active frontier

SG-000010 — Approved bounded argv `process.spawn`

Branch:
`feat/sg-000010-bounded-process-spawn`

SG-000010 may add exactly one new public EXECUTE capability:
- `process.spawn` / `spawn`
- executable and argv are separate typed fields; no raw command string
- cwd resolves inside the selected workspace
- caller environment overrides are absent
- stdin policy is null/no-input only
- timeout/stdout/stderr are bounded
- network class is `NONE`
- every execution requires fresh local approval bound to the exact normalized plan
- execution uses the existing AppContainer + Job containment path and verified termination semantics

Still denied / absent in SG-000010:
- `powershell.run`
- raw shell command strings
- process network capability
- background/detached execution
- public process kill capability
- browser automation
- Windows UI Automation/input injection
- elevation
- Git mutation

The next P05 unit after SG-000010 must be derived from the canonical delivery plan and SG-000010 closeout evidence; bounded PowerShell and further protected-state hardening remain successor concerns.

Architecture: Cotra is standalone; Kernux is not a dependency.

Evidence rule: never claim PROVEN or CLOSED without the required exact-head, platform, merge, post-merge, and governance evidence applicable to that grain.

Language rule: all repository technical content is English only.
