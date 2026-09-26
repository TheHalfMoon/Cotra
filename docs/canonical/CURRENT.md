# Cotra Current Canonical Frontier

Status: IMPLEMENTATION
Date: 2026-09-26
Governance snapshot base: 9c1e932d93abd46943e91af1118df0dce5fe35de
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

## SG-000010 activation

The SG-000010 governance/spec activation merged as `9c1e932d93abd46943e91af1118df0dce5fe35de` from exact qualified head `9c3824d53dd25047491cc204ac87b6eb333fb785`.

Activation evidence:
- exact-head CI `36240040249` — SUCCESS;
- exact-head Review Gates `36240040252` — SUCCESS;
- genuine Jev — 4/4 hunks, zero findings, zero blocking findings;
- Alibaba Open Code Review delegation — SUCCESS;
- activation post-merge CI `36241334989` — 5/5 SUCCESS.

Activation added no runtime authority. It authorized the bounded implementation grain only.

## Canonical capabilities before SG-000010 implementation

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

Implementation branch:
`feat/sg-000010-implementation`

The implementation candidate adds exactly one public EXECUTE capability:
- `process.spawn` / `spawn`;
- executable and argv are separate typed fields; no raw command string;
- cwd resolves inside the selected workspace;
- caller environment overrides are absent;
- stdin policy is null/no-input only;
- timeout/stdout/stderr are bounded;
- network class is `NONE`;
- every execution requires fresh local approval bound to the exact normalized execution plan;
- execution reuses the existing zero-capability AppContainer + Job containment path and verified termination semantics.

The implementation is not PROVEN until exact-head native Windows, portable CI, Jev, Alibaba OCR, security review, merge, and post-merge evidence are complete.

Still denied / absent in SG-000010:
- `powershell.run`;
- raw shell command strings;
- caller-provided process environment overrides;
- process stdin payload injection;
- process network capability;
- background/detached execution;
- public process kill capability;
- browser automation;
- Windows UI Automation/input injection;
- elevation;
- Git mutation.

The next P05 unit after SG-000010 must be derived from the canonical delivery plan and SG-000010 closeout evidence. Bounded PowerShell and further protected-state hardening remain successor concerns, not current authority.

Architecture: Cotra is standalone; Kernux is not a dependency.

Evidence rule: never claim PROVEN or CLOSED without the required exact-head, platform, merge, post-merge, and governance evidence applicable to that grain.

Language rule: all repository technical content is English only.
