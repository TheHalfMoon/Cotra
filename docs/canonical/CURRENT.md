# Cotra Current Canonical Frontier

Status: IMPLEMENTATION
Date: 2026-09-24
Canonical main: ed5cd2be9acd7aac93754965d055c82f77a01884

SG-000001 through SG-000007 are COMPLETE_CANONICAL.

SG-000007:
- exact-head CI 35971171993 — SUCCESS
- merge ed5cd2be9acd7aac93754965d055c82f77a01884
- post-merge CI 35971323866 — SUCCESS
- native Windows Job Object create/configure/close lifecycle proven

Canonical capabilities:
- system.status
- workspace.get
- fs.stat/list/read/search
- approved fs.write preview/write
- git.status/diff/log
- secure tunnel-client supervisor with file-referenced runtime credential isolation
- internal protected execution planning contract
- native Windows AppContainer profile primitive qualified
- native Windows Job Object kill-on-close primitive qualified

Live ChatGPT Secure MCP Tunnel E2E remains UNPROVEN until exercised on a real Windows runtime with a real tunnel ID/runtime credential.

Active grain:
SG-000008 — Contained AppContainer child launch qualification

Branch:
feat/sg-000008-contained-launch-probe

SG-000008 must prove the sequence:
- zero-capability AppContainer child created suspended
- AppContainer token verified
- child assigned to kill-on-close Job Object
- Job membership verified before resume
- child resumed and observed to bounded completion
- profile/handles cleaned up

Still denied / absent:
- process.spawn MCP authority
- PowerShell
- arbitrary executable launch
- browser automation
- UI automation/input
- elevation

Architecture:
Cotra is standalone; Kernux is not a dependency.

Evidence rule:
Never claim PROVEN without required platform/test evidence.

Language rule:
All repository technical content is English only.
