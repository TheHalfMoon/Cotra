# Cotra Current Canonical Frontier

Status: IMPLEMENTATION
Date: 2026-09-24

## Canonical main

Current base for active work:
4e83e019d8f3ebc38d6bd36edbce1afdcac1a5b7

SG-000001 is merged and exact-head qualified. Post-merge CI run 35922412274 passed all five jobs, including native Windows Rust and Node jobs.

Canonical capabilities:
- system.status/get
- workspace.get/get
- fs.stat/stat
- fs.list/list
- fs.read/read
- fs.search/search

Canonical authority remains read-only.

## Active grain

SG-000002 — Approved UTF-8 file mutation

Branch:
feat/sg-000002-approved-file-write

Purpose:
- add file-write preview;
- add approved create/overwrite;
- bind approval to content/current-state digests;
- reject stale target state;
- keep approval outside the MCP tool surface.

## Architecture decision

Cotra is standalone.

Kernux is not a dependency and must not be modified as part of Cotra work unless the founder explicitly creates a future integration task.

## Evidence rule

Never claim a capability PROVEN unless its required test and platform evidence exists.

## Language rule

Repository content, code, comments, commands, reports, specs, PR bodies, reviewer responses, and other technical work are English only.
