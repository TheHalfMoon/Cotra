# Cotra Current Canonical Frontier

Status: GOVERNANCE_CLOSEOUT
Date: 2026-09-26
Governance snapshot base: ddfcb4c9d5f06af446d44e031eb96bd70e4424d9
Evidence ledger: `.specgrain/canonical-evidence.json`

`Governance snapshot base` records the canonical parent from which this snapshot was authored. It deliberately does not claim the eventual merge SHA of the commit containing this file.

## SpecGrain state semantics

- `GRAIN`: authorized/active packet whose acceptance is not yet canonically proven.
- `PROVEN`: acceptance and required qualification are proven, but canonical closeout/governance is incomplete.
- `CLOSED`: machine state corresponding to Diffcipline `COMPLETE_CANONICAL`.

The machine-readable evidence for every `CLOSED` grain is recorded in `.specgrain/canonical-evidence.json` and validated by CI.

## Closed canonical grains

SG-000001 through SG-000012 are `CLOSED` and canonical on the snapshot base.

SG-000013 is implemented, exact-head qualified, merged, and post-merge verified. This closeout candidate moves its machine state to `CLOSED` and records its canonical evidence. That state becomes canonically effective only after this governance-only closeout is exact-head qualified, merged normally, and passes post-merge CI.

## SG-000013 implementation evidence

- implementation PR: `#27`;
- implementation base: `6b4ec228619292271dd77700f0c7d966c219124c`;
- qualified implementation head: `74b4a91d7b8e7f582a0df0fc814526658795514f`;
- implementation exact-head CI: `36258473411` — 5/5 SUCCESS;
- implementation Review Gates: `36258472190` — SUCCESS;
- genuine TypeSafe Jev: 2/2 hunks, zero findings, zero blocking findings;
- Alibaba Open Code Review v1.12.9: exact-range delegation SUCCESS, 2 files total, 1 reviewable and 1 excluded Markdown security note manually reviewed;
- unresolved implementation review threads: 0;
- implementation merge: `ddfcb4c9d5f06af446d44e031eb96bd70e4424d9`;
- implementation post-merge CI: `36258601073` — 5/5 SUCCESS.

Native Windows qualification proved:
- the exact inbox Windows PowerShell executable can run inside the existing zero-capability AppContainer/Job containment boundary;
- deterministic bounded PowerShell success completed with expected stdout;
- NUL/no-input stdin was preserved;
- tested Cotra/tunnel/API secret-like environment values were excluded;
- timeout produced verified `ProcessTimeout` semantics;
- stdout and stderr limits produced verified stream-typed `OutputLimit` semantics;
- AppContainer identity, pre-resume Job membership, bounded output, and final Job quiescence remained verified;
- existing SG-000011 descendant lifecycle and SG-000012 protected-state/authority-channel isolation regressions remained green.

## Canonical public authority boundary retained

SG-000013 introduced no public authority.

Windows public `process.spawn` remains positively restricted to the exact SG-000010-qualified `%SystemRoot%\System32\whoami.exe` target.

Still denied or absent:
- public `powershell.run`;
- caller-provided PowerShell script text;
- direct PowerShell executable authority through `process.spawn`;
- generic executable-registry widening;
- additional public `process.spawn` executable targets;
- `cmd.exe` / raw shell authority;
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

## Next frontier

No successor grain is activated by this closeout candidate.

After SG-000013 becomes canonically `CLOSED`, repository truth must determine the next COTRA-P05 unit. The canonical P05 plan requires bounded PowerShell, but public `powershell.run` must not be exposed merely because provider-private/native qualification succeeded. A successor may expose a separately approved public bounded PowerShell capability only if the roadmap, policy model, approval binding, containment evidence, and current canonical records support that authority delta without weakening the proven process boundary. Otherwise additional containment evidence must be completed first.

Architecture: Cotra is standalone; Kernux is not a dependency.

Evidence rule: never claim PROVEN or CLOSED without required exact-head, platform, merge, post-merge, and governance evidence.

Language rule: all repository technical content is English only.
