# Cotra Current Canonical Frontier

Status: GOVERNANCE_RECONCILIATION
Date: 2026-09-26
Governance snapshot base: 4d843a6b519faf39ac16e320e99683b236140b5c
Evidence ledger: `.specgrain/canonical-evidence.json`

`Governance snapshot base` records the canonical parent from which this snapshot was authored. It deliberately does not claim the eventual merge SHA of the commit containing this file. This avoids the self-referential stale-main failure mode that affected earlier closeout snapshots.

## SpecGrain state semantics

- `GRAIN`: authorized/active packet whose acceptance is not yet canonically proven.
- `PROVEN`: acceptance and required qualification are proven, but canonical closeout/governance is incomplete.
- `CLOSED`: machine state corresponding to Diffcipline `COMPLETE_CANONICAL`.

The machine-readable evidence for every `CLOSED` grain is recorded in `.specgrain/canonical-evidence.json` and validated by CI.

## Closed canonical grains

SG-000001 through SG-000009A are `CLOSED` in this reconciliation candidate. The state becomes canonical only when this governance-only reconciliation is merged with exact-head qualification and post-merge verification.

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

## Still denied or absent

- public/MCP `process.spawn` authority
- PowerShell
- caller-selected arbitrary executable authority
- process network authority
- browser automation
- Windows UI Automation/input injection
- elevation
- Git mutation

## Active frontier

Canonical-state reconciliation only.

No successor grain is canonical yet. After this reconciliation merges and post-merge CI succeeds, the next lawful unit must be derived from the canonical architecture/delivery plan, remaining P05 exit criteria, and repository truth. Local or untracked SG-000010 artifacts are not canonical evidence.

Architecture: Cotra is standalone; Kernux is not a dependency.

Evidence rule: never claim PROVEN or CLOSED without the required exact-head, platform, merge, post-merge, and governance evidence applicable to that grain.

Language rule: all repository technical content is English only.
