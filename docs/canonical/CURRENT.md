# Cotra Current Canonical Frontier

Status: GOVERNANCE_CLOSEOUT
Date: 2026-09-26
Governance snapshot base: 6b24a810712c7ee1edfab714c672447606dc0b13
Evidence ledger: `.specgrain/canonical-evidence.json`

`Governance snapshot base` records the canonical parent from which this snapshot was authored. It deliberately does not claim the eventual merge SHA of the commit containing this file.

## SpecGrain state semantics

- `GRAIN`: authorized/active packet whose acceptance is not yet canonically proven.
- `PROVEN`: acceptance and required qualification are proven, but canonical closeout/governance is incomplete.
- `CLOSED`: machine state corresponding to Diffcipline `COMPLETE_CANONICAL`.

The machine-readable evidence for every `CLOSED` grain is recorded in `.specgrain/canonical-evidence.json` and validated by CI.

## Closed canonical grains

SG-000001 through SG-000011 are `CLOSED` and canonical on the snapshot base.

SG-000012 is implemented, exact-head qualified, merged, and post-merge verified. This closeout candidate moves its machine state to `CLOSED` and records its canonical evidence. That state becomes canonically effective only after this governance-only closeout is exact-head qualified, merged normally, and passes post-merge CI.

## SG-000012 implementation evidence

- implementation PR: `#24`;
- implementation base: `fb84a7fe8bab9589f29213eecdd346a306910fec`;
- qualified implementation head: `2fa87c15a3a6f7d0fdc3cf06ed3015ccce0a039e`;
- implementation exact-head CI: `36255254224` — 5/5 SUCCESS;
- implementation Review Gates: `36255253042` — SUCCESS;
- genuine TypeSafe Jev: 2/2 hunks, zero findings, zero blocking findings;
- Alibaba Open Code Review v1.12.9: exact-range delegation SUCCESS, 2 files total, 1 reviewable, 1 excluded security note manually reviewed;
- implementation merge: `6b24a810712c7ee1edfab714c672447606dc0b13`;
- implementation post-merge CI: `36255431331` — 5/5 SUCCESS, including native Windows;
- unresolved implementation review threads: 0.

Qualified SG-000012 behavior:
- a zero-capability AppContainer child could not read the Cotra-owned LocalAppData protected-state sentinel;
- tested Cotra/API secret-like variables were absent from the contained child environment;
- contained stdin was NUL/EOF rather than the cotra-mcp -> cotrad authority channel;
- AppContainer identity, pre-resume Job membership, and final Job quiescence remained verified;
- no Cotra-owned ACL repair was required;
- no trusted workspace or arbitrary user-file ACL was modified;
- SG-000009, SG-000009A, SG-000010, and SG-000011 regressions remained green.

## Canonical public process boundary retained

No public authority widened in SG-000012.

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

SG-000012 canonical closeout only.

No successor authority is canonical yet. After this closeout merges and its post-merge CI succeeds, repository truth must determine the next lawful COTRA-P05 unit from the canonical architecture/delivery plan and remaining P05 exit criteria. Likely candidates include positive executable-registry widening or bounded PowerShell, but neither authority is granted by this closeout.

Architecture: Cotra is standalone; Kernux is not a dependency.

Evidence rule: never claim PROVEN or CLOSED without required exact-head, platform, merge, post-merge, and governance evidence.

Language rule: all repository technical content is English only.
