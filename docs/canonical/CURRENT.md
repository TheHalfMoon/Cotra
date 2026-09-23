# Cotra Current Canonical Frontier

Status: IMPLEMENTATION
Date: 2026-09-24
Canonical base: 7d4003a1b6a82cac577c77e91b7d2606f49733f4

SG-000001 and SG-000002 are COMPLETE_CANONICAL.
SG-000002 exact-head CI: 35923989601 (5/5 SUCCESS).
SG-000002 post-merge CI: 35924151711 (5/5 SUCCESS).

Canonical capabilities: system.status, workspace.get, fs.stat/list/read/search, and approved fs.write preview/write.

Active grain: SG-000003 — Read-only Git provider
Branch: feat/sg-000003-read-only-git
Purpose: typed git.status/diff/log with fixed local Git invocation, bounded output, sanitized child environment, and no network or generic shell authority.

Architecture: Cotra is standalone; Kernux is not a dependency.
Evidence rule: never claim PROVEN without required platform/test evidence.
Language rule: all repository technical content is English only.
