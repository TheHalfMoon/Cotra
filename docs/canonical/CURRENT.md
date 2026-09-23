# Cotra Current Canonical Frontier

Status: IMPLEMENTATION
Date: 2026-09-24

## Canonical main

Current base for active work:
7d4003a1b6a82cac577c77e91b7d2606f49733f4

SG-000001 and SG-000002 are COMPLETE_CANONICAL.

SG-000002 evidence:
- exact-head CI run 35923989601: 5/5 SUCCESS;
- merge SHA: 7d4003a1b6a82cac577c77e91b7d2606f49733f4;
- post-merge CI run 35924151711: 5/5 SUCCESS.

Canonical capabilities:
- system.status/get
- workspace.get/get
- fs.stat/stat
- fs.list/list
- fs.read/read
- fs.search/search
- fs.write/preview
- fs.write/write with independent local approval

## Active grain

SG-000003 — Read-only Git provider

Branch:
feat/sg-000003-read-only-git

Purpose:
- complete COTRA-P02 read-only Git;
- expose typed status/diff/log only;
- keep Git invocation internal and fixed;
- sanitize the child environment;
- prevent network or generic shell authority.

## Architecture decision

Cotra is standalone.

Kernux is not a dependency and must not be modified as part of Cotra work unless the founder explicitly creates a future integration task.

## Evidence rule

Never claim a capability PROVEN unless its required test and platform evidence exists.

## Language rule

Repository content, code, comments, commands, reports, specs, PR bodies, reviewer responses, and other technical work are English only.
