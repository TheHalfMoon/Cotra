# Cotra Current Canonical Frontier

Status: PLANNING
Date: 2026-09-23

## Canonical main

Base SHA:
7de4779682b9e8dc794d6c675d7ed6c420355d13

Main currently contains only the project bootstrap README.

No privileged implementation is canonical.

## Active planning branch

plan/initial-architecture-2026-09-23

This branch defines:
- standalone Cotra architecture;
- OpenAI Secure MCP Tunnel topology;
- Windows security boundaries;
- source/provenance ledger;
- threat model;
- Diffcipline;
- Apache-2.0 project license;
- SG-000001.

## Architecture decision

Cotra is a standalone project.

Kernux is not a dependency and must not be modified as part of Cotra work unless the founder explicitly creates a future integration task.

## Implementation authority

Implementation is NOT yet canonical.

The first implementation packet after planning acceptance is:

SG-000001 — Trusted read-only MCP path and policy skeleton

SG-000001 intentionally excludes:
- file mutation;
- shell/PowerShell execution;
- Git mutation;
- browser control;
- clipboard access;
- Windows UI Automation;
- mouse/keyboard injection;
- elevation;
- donor code import.

## Next lawful sequence

1. Review and merge the planning PR.
2. Reverify exact live main after merge.
3. Open SG-000001 implementation branch from exact canonical main.
4. Implement only the bounded read-only slice.
5. Run exact-head CI plus Windows-native qualification.
6. Run Diffcipline, Jev-style semantic review, Alibaba Open Code Review, and security review.
7. Merge only a qualified exact head under governance.
8. Verify post-merge canonical state before advancing authority.

## Evidence rule

Never claim a capability PROVEN unless its required test and platform evidence exists.

A tunnel or product integration that cannot be exercised must remain UNPROVEN rather than being inferred from documentation.

## Language rule

Repository content, code, comments, commands, reports, specs, PR bodies, reviewer responses, and other technical work are English only.
