# Cotra Current Canonical Frontier

Status: GOVERNANCE_CLOSEOUT
Date: 2026-09-26
Governance snapshot base: 388f5de196580d71275a31b84b584f5ac280baec
Evidence ledger: `.specgrain/canonical-evidence.json`

`Governance snapshot base` records the canonical parent from which this snapshot was authored. It deliberately does not claim the eventual merge SHA of the commit containing this file.

## SpecGrain state semantics

- `GRAIN`: authorized/active packet whose acceptance is not yet canonically proven.
- `PROVEN`: acceptance and required qualification are proven, but canonical closeout/governance is incomplete.
- `CLOSED`: machine state corresponding to Diffcipline `COMPLETE_CANONICAL`.

The machine-readable evidence for every `CLOSED` grain is recorded in `.specgrain/canonical-evidence.json` and validated by CI.

## Closed canonical grains before this candidate

SG-000001 through SG-000009A are `CLOSED` and canonical after governance reconciliation merge `be1e6a28c77d25a8f9f1848c02df290ac74da734` and post-merge CI `36239819409` (5/5 SUCCESS).

Historical boundaries remain explicit:
- SG-000004 closed tunnel supervisor/credential isolation only; live ChatGPT Secure MCP Tunnel E2E remains UNPROVEN until exercised on a real Windows runtime with a real tunnel ID/runtime credential.
- SG-000009 closed the fixed provider-private no-argument `whoami.exe` AppContainer success path.
- SG-000009A closed provider-private destructive timeout/stdout-limit/stderr-limit qualification with verified zero active Job processes and fail-closed `TerminationUnverified` semantics.

## SG-000010 closeout candidate

SG-000010 — Approved bounded argv `process.spawn`

Implementation evidence:
- activation merge: `9c1e932d93abd46943e91af1118df0dce5fe35de`;
- qualified implementation head: `248b96c17e3dc387371441ba12b5ee75d41c2495`;
- exact-head CI: `36248970994` — SUCCESS;
- exact-head Review Gates: `36248969775` — SUCCESS;
- genuine TypeSafe Jev: 46/46 hunks, zero findings, zero blocking findings;
- Alibaba Open Code Review v1.12.9: exact-range delegation SUCCESS, 17 files total / 10 reviewable / 7 excluded;
- exact-head security re-review: PASS after the arbitrary-executable authority defect was repaired with a positive executable policy;
- implementation merge: `388f5de196580d71275a31b84b584f5ac280baec`;
- implementation post-merge CI: `36250111765` — 5/5 SUCCESS, including native Windows.

This governance candidate changes SG-000010 to `CLOSED` and records its canonical evidence. That state becomes canonical only if this closeout candidate itself merges normally and its post-merge CI succeeds.

## Canonical capability added by SG-000010

One bounded public EXECUTE capability exists:
- `process.spawn` / `spawn`;
- executable and argv are separate typed fields; raw command strings are absent;
- on Windows, executable authority is positively restricted to the exact qualified `%SystemRoot%\System32\whoami.exe` target;
- cwd resolves inside the selected trusted workspace;
- caller environment overrides are absent;
- stdin is null/no-input only;
- timeout/stdout/stderr are bounded;
- `network_class=NONE` only;
- every execution requires fresh local approval bound to the normalized execution plan;
- execution uses the zero-capability AppContainer + Job containment path;
- destructive timeout/output-limit classifications require verified Job quiescence; inability to prove termination fails closed as `TerminationUnverified`.

## Security boundary retained

Still denied or absent:
- generic executable-registry authority;
- `cmd.exe` or raw shell execution;
- `powershell.run` and direct PowerShell executable authority;
- caller-provided process environment overrides;
- process stdin payload injection;
- process network authority;
- detached/background execution;
- public process kill capability;
- Git mutation;
- browser automation;
- Windows UI Automation/input injection;
- elevation;
- approval bypass or persistent approval reuse.

The contained executor's generic post-primary-exit pipe-drain path remains unqualified for arbitrary descendant-producing executables. Descendant-aware post-exit lifecycle hardening is mandatory before executable-registry authority can widen.

## Active frontier

SG-000010 governance closeout only.

No successor P05 grain is canonical yet. After this closeout merges and post-merge CI succeeds, derive the next lawful unit from the canonical delivery plan and the retained SG-000010 limitations. Candidate successor concerns include descendant-aware lifecycle hardening, executable-registry widening, bounded PowerShell, and protected-state hardening; none is authorized by this closeout alone.

Architecture: Cotra is standalone; Kernux is not a dependency.

Evidence rule: never claim PROVEN or CLOSED without required exact-head, platform, merge, post-merge, and governance evidence.

Language rule: all repository technical content is English only.
