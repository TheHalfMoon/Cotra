# Cotra Current Canonical Frontier

Status: IMPLEMENTATION
Date: 2026-09-24
Canonical main: 2524b8f8bbf3ff196cb1bc2915312e785ab7a858

SG-000001 through SG-000008 are COMPLETE_CANONICAL.

SG-000008:
- exact-head CI 36017984916 ? SUCCESS
- Review Gates 36017981120 ? SUCCESS
- merge 2524b8f8bbf3ff196cb1bc2915312e785ab7a858
- post-merge CI 36018639615 ? SUCCESS
- native Windows AppContainer suspended launch, token verification, Job membership before resume, bounded completion, and cleanup proven

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
- native Windows contained fixed-child launch qualification

Live ChatGPT Secure MCP Tunnel E2E remains UNPROVEN until exercised on a real Windows runtime with a real tunnel ID/runtime credential.

Active grain:
SG-000009 ? Private bounded argv execution qualification

Branch:
feat/sg-000009-private-contained-argv

SG-000009 must prove:
- fixed private `whoami.exe` execution without arguments without shell interpolation
- explicit absolute executable plus validated workspace-bound cwd applied to CreateProcessW
- scrubbed environment and an explicit three-handle allowlist: NUL stdin, stdout, stderr
- AppContainer token and Job membership verification before resume
- bounded stdout/stderr capture, exit code, and bounded completion
- zero active Job processes before success

Deferred to successor work:
- destructive timeout/output-limit runtime qualification
- descendant-tree termination fixtures
- any caller-selected process authority or configured workspace grant

Still denied / absent:
- process.spawn MCP authority
- PowerShell
- caller-selected arbitrary executable authority
- network authority
- browser automation
- UI automation/input
- elevation

Architecture:
Cotra is standalone; Kernux is not a dependency.

Evidence rule:
Never claim PROVEN without required platform/test evidence.

Language rule:
All repository technical content is English only.
