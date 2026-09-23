# Cotra

**Computer Orchestration & Trusted Runtime Access**

Cotra is an open-source, local-first MCP gateway for securely connecting ChatGPT and other authorized MCP clients to a Windows computer without exposing a general-purpose remote-control endpoint to the public internet.

## Status

Bootstrap / architecture planning.

No privileged automation implementation is considered production-ready yet.

## Product goal

Cotra should provide a safer, more capable alternative to unrestricted desktop-command MCP servers by separating:

- MCP transport and client compatibility
- local policy and approval authority
- filesystem and process capabilities
- Git workflows
- browser automation
- Windows application and UI automation
- audit and evidence
- secrets and network egress

## Core principles

1. **Local-first.** Privileged operations execute on the user's computer.
2. **Private by default.** Prefer outbound-only connectivity such as OpenAI Secure MCP Tunnel rather than public inbound ports.
3. **Least authority.** Expose bounded capabilities instead of a universal `run_anything` primitive.
4. **Structured before visual.** Prefer typed APIs, filesystem APIs, process APIs, Git, browser DOM/accessibility, and Windows UI Automation before coordinate-based control.
5. **Human approval for consequential actions.** The agent must not be able to approve its own protected action.
6. **Workspace isolation.** File, process, browser, and Git actions are bound to explicit workspace/session identities.
7. **Evidence before success.** A tool result must distinguish requested, started, completed, verified, failed, cancelled, and indeterminate outcomes.
8. **Auditable.** Security-sensitive operations produce structured local audit records with secret redaction.
9. **Zero founder-funded runtime infrastructure.** The default architecture should not require Cotra to operate paid hosted infrastructure.
10. **Windows first, portable contracts.** Windows is the first-class host target while the capability contracts remain portable enough for future adapters.

## Intended capability families

- system and workspace observation
- file read/search/write
- PowerShell and process execution
- Git inspection and mutation
- application/window observation
- Windows UI Automation
- screenshots and visual fallback
- browser automation
- clipboard access
- bounded network access
- local approvals
- local audit and evidence

## Non-goals

Cotra is not intended to be:

- an unauthenticated remote shell
- a public desktop-control endpoint
- a credential exfiltration bridge
- an agent that can silently approve its own protected actions
- a replacement for operating-system security boundaries

## Governance

Architecture, source provenance, security controls, and implementation slices are planned before privileged implementation begins. Work is intended to use bounded SpecGrain work packets, exact-diff review discipline, independent review, and explicit evidence gates.

