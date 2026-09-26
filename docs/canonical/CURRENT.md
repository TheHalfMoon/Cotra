# Cotra Current Canonical Frontier

Status: IMPLEMENTATION
Date: 2026-09-26
Governance snapshot base: 54b5b8ff39fe4188f09950d1db5c42e8823b19a6
Evidence ledger: `.specgrain/canonical-evidence.json`

`Governance snapshot base` records the canonical parent from which this snapshot was authored. It deliberately does not claim the eventual merge SHA of the commit containing this file.

## SpecGrain state semantics

- `GRAIN`: authorized/active packet whose acceptance is not yet canonically proven.
- `PROVEN`: acceptance and required qualification are proven, but canonical closeout/governance is incomplete.
- `CLOSED`: machine state corresponding to Diffcipline `COMPLETE_CANONICAL`.

The machine-readable evidence for every `CLOSED` grain is recorded in `.specgrain/canonical-evidence.json` and validated by CI.

## Closed canonical grains

SG-000001 through SG-000011 are `CLOSED` and canonical.

SG-000011 closeout evidence:
- qualified implementation head: `b6ca7106196e3d1c4771f46f311ec22a413eff49`;
- implementation merge: `00cab385bdce43562530646dce84be71e1ddfabd`;
- implementation exact-head CI: `36251790829` — 5/5 SUCCESS;
- implementation Review Gates: `36251789043` — genuine Jev 11/11 hunks, zero findings/blocking findings; Alibaba Open Code Review exact-range delegation SUCCESS;
- implementation post-merge CI: `36253841733` — 5/5 SUCCESS;
- closeout exact head: `6797b994f937df46cdbda1720a7cd6a3fac57878`;
- closeout exact-head CI: `36254155605` — 5/5 SUCCESS;
- closeout Review Gates: `36254155073` — genuine Jev 7/7 hunks, zero findings/blocking findings; Alibaba Open Code Review exact-range delegation SUCCESS;
- closeout merge: `54b5b8ff39fe4188f09950d1db5c42e8823b19a6`;
- closeout post-merge CI: `36254286169` — 5/5 SUCCESS.

Canonical public process authority remains narrow:
- `process.spawn` / `spawn` is argv-first;
- Windows executable authority remains positively restricted to the exact SG-000010-qualified `%SystemRoot%\System32\whoami.exe` target;
- cwd is workspace-bound;
- caller environment overrides and stdin payloads are absent;
- timeout/stdout/stderr are bounded;
- `network_class=NONE` only;
- every execution requires fresh local approval bound to the normalized plan;
- execution uses zero-capability AppContainer containment, pre-resume Job membership, bounded post-primary-exit drain, and verified termination semantics.

Historical boundary: SG-000004 closed tunnel supervisor/credential isolation only; live ChatGPT Secure MCP Tunnel E2E remains UNPROVEN until exercised on a real Windows runtime with a real tunnel ID/runtime credential.

## Active frontier

SG-000012 — Restricted-child protected-state isolation qualification

Branch:
`feat/sg-000012-protected-state-isolation`

This is the next lawful COTRA-P05 unit because the canonical P05 exit criteria require Cotra protected state and the current authority channel to remain inaccessible to a restricted child where the architecture supports that proof. Broader executable or PowerShell authority must not be introduced before this boundary is qualified.

SG-000012 must prove on native Windows:
- a provider-private zero-capability AppContainer child cannot read a Cotra-owned protected-state sentinel under the parent user's LocalAppData Cotra state boundary;
- child environment inheritance remains free of Cotra/tunnel secret-like variables;
- child handle inheritance remains limited to NUL stdin plus dedicated stdout/stderr handles;
- the current cotra-mcp to cotrad stdio authority channel is not inherited by process.spawn children;
- if the default Cotra-owned state ACL boundary is insufficient, only Cotra-owned state is hardened and native requalification proves denial;
- no user workspace or arbitrary user-file ACL is modified;
- SG-000009, SG-000009A, SG-000010, and SG-000011 regressions remain green.

## Authority boundary retained during SG-000012

No new public process, filesystem, network, browser/UI, Git, approval, or privileged authority is authorized by this grain.

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

Only after SG-000012 is `CLOSED` may repository truth determine the next P05 authority unit, including positive executable-registry widening or bounded PowerShell qualification.

Architecture: Cotra is standalone; Kernux is not a dependency.

Evidence rule: never claim PROVEN or CLOSED without required exact-head, platform, merge, post-merge, and governance evidence.

Language rule: all repository technical content is English only.
