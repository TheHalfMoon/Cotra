# Cotra Current Canonical Frontier

Status: IMPLEMENTATION
Date: 2026-09-24
Canonical main: 0f71ca47b32f08045f83d194497aa49daad2d4de

SG-000001 through SG-000005 are merged.

SG-000004:
- exact-head CI 35969551340 — 5/5 SUCCESS
- merge 4aa28b128a27d21cf6fa768ca34d44db4b3b39cc
- post-merge CI 35969805332 — 5/5 SUCCESS
- COMPLETE_CANONICAL

SG-000005:
- exact-head CI 35970146506 — 5/5 SUCCESS
- merge 0f71ca47b32f08045f83d194497aa49daad2d4de
- post-merge CI 35970291619 — 5/5 SUCCESS
- COMPLETE_CANONICAL

Canonical capabilities:
- system.status
- workspace.get
- fs.stat/list/read/search
- approved fs.write preview/write
- git.status/diff/log
- secure tunnel-client supervisor with file-referenced runtime credential isolation
- internal protected execution planning contract (not externally exposed)

Live ChatGPT Secure MCP Tunnel E2E remains UNPROVEN until exercised on a real Windows runtime with a real tunnel ID/runtime credential.

Active grain:
SG-000006 — Windows AppContainer primitive qualification
Branch:
feat/sg-000006-appcontainer-probe

SG-000006 qualifies only the Windows AppContainer profile lifecycle. It launches no child process and exposes no new MCP authority.

Still denied / absent:
- process.spawn
- PowerShell
- browser automation
- UI automation/input
- elevation

Architecture:
Cotra is standalone; Kernux is not a dependency.

Evidence rule:
Never claim PROVEN without required platform/test evidence.

Language rule:
All repository technical content is English only.
