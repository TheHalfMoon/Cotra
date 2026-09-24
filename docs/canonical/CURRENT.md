# Cotra Current Canonical Frontier

Status: IMPLEMENTATION
Date: 2026-09-24
Canonical base: 527a5e69e453fa740a754c4f037dbb1b2764c917

SG-000001 through SG-000003 are COMPLETE_CANONICAL.
SG-000003 exact-head CI: 35924850354 (5/5 SUCCESS).
SG-000003 post-merge CI: 35924987328 (5/5 SUCCESS).

Active grain: SG-000004 — Secure MCP Tunnel supervisor and credential isolation
Branch: feat/sg-000004-tunnel-supervisor

Security decision: openai/tunnel-client stdio MCP children inherit the tunnel-client environment. Cotra therefore forbids runtime-key environment delivery and uses control-plane.api-key=file:<path>, with the key file outside trusted workspaces.

Live ChatGPT tunnel E2E remains UNPROVEN until exercised with a real tunnel/runtime key on Windows.

Architecture: Cotra is standalone; Kernux is not a dependency.
Evidence rule: never claim PROVEN without required platform/test evidence.
Language rule: all repository technical content is English only.
