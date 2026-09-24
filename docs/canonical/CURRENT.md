# Cotra Current Canonical Frontier

Status: IMPLEMENTATION
Date: 2026-09-24
Canonical main: b350932c4226fc5250a2d234c1927149f7c49064

SG-000009 closeout merge (historical): 617b06ca234298f5cca184d66c04df5ce1b1b4cc

The canonical main line above records the current repository head after SG-000009A governance activation. It is intentionally distinct from the historical SG-000009 closeout merge above.

SG-000001 through SG-000009 are COMPLETE_CANONICAL.

SG-000009:
- qualified head ca17e5b5f8145dfb01ad0ad6102587ef7f80ed76
- implementation merge ba6aa919d35432db0c049ca970be29f95767041f
- closeout merge 617b06ca234298f5cca184d66c04df5ce1b1b4cc
- pre-merge CI 36027271683 - SUCCESS
- Review Gates 36027269455 - SUCCESS; Jev 20/20 hunks, zero findings; OCR exact range PASSED
- post-merge CI 36029004410 - SUCCESS
- SG-000009 closeout post-merge CI 36029741014 - SUCCESS
- native Windows fixed no-argument whoami.exe AppContainer qualification proven

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
SG-000009A - Destructive timeout and output-limit termination qualification

Branch:
feat/sg-000009a-implementation

SG-000009A must prove:
- fixed provider-private timeout and output-limit child modes
- Job termination with verified zero active processes
- typed ProcessTimeout, OutputLimit, and TerminationUnverified evidence
- no expansion of process.spawn, PowerShell, arbitrary executable, network, ACL, browser/UI, elevation, or approval authority

SG-000009 remains canonical for the fixed private no-argument child success path. Destructive timeout/output-limit descendant-tree runtime qualification is the next lawful P05 unit.

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
