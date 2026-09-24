# Cotra Current Canonical Frontier

Status: IMPLEMENTATION
Date: 2026-09-24
Canonical base for active work: 4aa28b128a27d21cf6fa768ca34d44db4b3b39cc

SG-000001 through SG-000004 are COMPLETE_CANONICAL.

SG-000004 exact-head CI:
- run 35969551340
- 5/5 SUCCESS on Windows/Ubuntu Node, Windows/Ubuntu Rust, and Governance

SG-000004 post-merge CI:
- run 35969805332
- 5/5 SUCCESS
- SG-000004 is COMPLETE_CANONICAL.

Canonical implemented capabilities before SG-000005:
- system.status
- workspace.get
- fs.stat/list/read/search
- approved fs.write preview/write
- git.status/diff/log
- secure tunnel-client supervisor with file-referenced runtime credential isolation

Live ChatGPT Secure MCP Tunnel E2E remains UNPROVEN until exercised on a real Windows runtime with a real tunnel ID/runtime credential.

Active grain:
SG-000005 — Protected execution foundation
Branch:
feat/sg-000005-protected-execution-foundation

SG-000005 intentionally exposes no process.spawn or PowerShell MCP tool. It creates the internal execution contract and containment proof boundary that must exist before EXECUTE authority is enabled.

Architecture:
Cotra is standalone; Kernux is not a dependency.

Evidence rule:
Never claim PROVEN without required platform/test evidence.

Language rule:
All repository technical content is English only.
