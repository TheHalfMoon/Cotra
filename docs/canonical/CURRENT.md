# Cotra Current Canonical Frontier

Status: IMPLEMENTATION
Date: 2026-09-24
Canonical main: fe3c5453b1b0b44b2e6209e3087c197e4a78c907

SG-000001 through SG-000006 are COMPLETE_CANONICAL.

SG-000006:
- exact-head CI 35970657031 — 5/5 SUCCESS
- merge fe3c5453b1b0b44b2e6209e3087c197e4a78c907
- post-merge CI 35970806759 — 5/5 SUCCESS
- native Windows AppContainer create/derive/delete lifecycle proven

Canonical capabilities:
- system.status
- workspace.get
- fs.stat/list/read/search
- approved fs.write preview/write
- git.status/diff/log
- secure tunnel-client supervisor with file-referenced runtime credential isolation
- internal protected execution planning contract
- native Windows AppContainer profile primitive qualified

Live ChatGPT Secure MCP Tunnel E2E remains UNPROVEN until exercised on a real Windows runtime with a real tunnel ID/runtime credential.

Active grain:
SG-000007 — Windows Job Object lifecycle qualification
Branch:
feat/sg-000007-job-object-probe

SG-000007 qualifies Job Object creation, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE configuration, and handle cleanup. It launches no child process and exposes no new MCP authority.

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
