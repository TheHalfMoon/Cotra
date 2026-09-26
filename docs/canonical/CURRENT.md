# Cotra Current Canonical Frontier

Status: GOVERNANCE_CLOSEOUT
Date: 2026-09-26
Governance snapshot base: 00cab385bdce43562530646dce84be71e1ddfabd
Evidence ledger: `.specgrain/canonical-evidence.json`

`Governance snapshot base` records the canonical parent from which this snapshot was authored. It deliberately does not claim the eventual merge SHA of the commit containing this file.

## SpecGrain state semantics

- `GRAIN`: authorized/active packet whose acceptance is not yet canonically proven.
- `PROVEN`: acceptance and required qualification are proven, but canonical closeout/governance is incomplete.
- `CLOSED`: machine state corresponding to Diffcipline `COMPLETE_CANONICAL`.

The machine-readable evidence for every `CLOSED` grain is recorded in `.specgrain/canonical-evidence.json` and validated by CI.

## Closed canonical grains

SG-000001 through SG-000010 are `CLOSED` and canonical on the snapshot base.

SG-000011 is implemented, exact-head qualified, merged, and post-merge verified. This closeout candidate moves its machine state to `CLOSED` and records its canonical evidence. That state becomes canonically effective only after this governance-only closeout is exact-head qualified, merged normally, and passes post-merge CI.

## SG-000011 implementation evidence

- implementation PR: `#21`;
- implementation base: `57bb970c69f55c7775900f47fe1047c5ff7322f7`;
- qualified implementation head: `b6ca7106196e3d1c4771f46f311ec22a413eff49`;
- implementation exact-head CI: `36251790829` — 5/5 SUCCESS;
- implementation Review Gates: `36251789043` — SUCCESS;
- genuine TypeSafe Jev: 11/11 hunks, zero findings, zero blocking findings;
- Alibaba Open Code Review v1.12.9: exact-range delegation SUCCESS, 2 files total, 1 reviewable, 1 excluded security note manually reviewed;
- implementation merge: `00cab385bdce43562530646dce84be71e1ddfabd`;
- implementation post-merge CI: `36253841733` — 5/5 SUCCESS, including native Windows;
- unresolved implementation review threads: 0.

Qualified SG-000011 behavior:
- post-primary-exit stdout/stderr drain is bounded;
- a retained-handle descendant fixture cannot hang the contained executor;
- active Job members beyond the bounded drain window trigger Job termination;
- zero active Job processes are verified before final success/destructive classification;
- inability to prove termination, quiescence, or terminal pipe state returns `TerminationUnverified`;
- SG-000009, SG-000009A, and SG-000010 regressions remain green.

## Canonical public process boundary retained

No public authority widened in SG-000011.

Windows public `process.spawn` remains positively restricted to the exact SG-000010-qualified `%SystemRoot%\System32\whoami.exe` target.

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

Historical boundary: SG-000004 closed tunnel supervisor/credential isolation only; live ChatGPT Secure MCP Tunnel E2E remains UNPROVEN until exercised on a real Windows runtime with a real tunnel ID/runtime credential.

## Active frontier

SG-000011 canonical closeout only.

No successor authority is canonical yet. After this closeout merges and its post-merge CI succeeds, repository truth must determine the next lawful P05 unit from the canonical architecture/delivery plan and remaining P05 exit criteria. The successor may address positive executable-registry widening, bounded PowerShell, protected-state/IPC hardening, or another prerequisite, but no such authority is granted by this closeout.

Architecture: Cotra is standalone; Kernux is not a dependency.

Evidence rule: never claim PROVEN or CLOSED without required exact-head, platform, merge, post-merge, and governance evidence.

Language rule: all repository technical content is English only.
