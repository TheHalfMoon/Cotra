use cotra_approval::{ApprovalBroker, ApprovalDecision, ApprovalPrompt, LocalApprovalBroker};
use cotra_audit::{default_audit_path, AuditLogger};
use cotra_contracts::{FailureCode, RequestEnvelope, ResponseEnvelope, INTERNAL_PROTOCOL_VERSION};
use cotra_policy::{PolicyEngine, Workspace, POLICY_REVISION};
use cotra_provider_fs::{FsProvider, ProviderError};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct WorkspaceConfig {
    id: String,
    root: PathBuf,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("cotrad fatal: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let policy = load_policy()?;
    let audit = AuditLogger::new(default_audit_path())
        .map_err(|error| format!("initialize audit log: {error}"))?;
    let approval = LocalApprovalBroker;

    eprintln!(
        "cotrad ready: protocol={} audit={} mode=SG-000002_APPROVED_WRITE",
        INTERNAL_PROTOCOL_VERSION,
        audit.path().display()
    );

    let stdin = io::stdin();
    let mut stdout = io::BufWriter::new(io::stdout().lock());

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("cotrad stdin error: {error}");
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<RequestEnvelope>(&line) {
            Ok(request) => handle_request(&policy, &audit, &approval, request),
            Err(error) => ResponseEnvelope::failure(
                "unknown",
                FailureCode::InvalidRequest,
                format!("invalid request JSON: {error}"),
            ),
        };

        serde_json::to_writer(&mut stdout, &response)
            .map_err(|error| format!("serialize response: {error}"))?;
        stdout
            .write_all(b"\n")
            .map_err(|error| format!("write response: {error}"))?;
        stdout
            .flush()
            .map_err(|error| format!("flush response: {error}"))?;
    }

    Ok(())
}

fn load_policy() -> Result<PolicyEngine, String> {
    let workspaces = if let Ok(raw) = std::env::var("COTRA_WORKSPACES_JSON") {
        let configs: Vec<WorkspaceConfig> = serde_json::from_str(&raw)
            .map_err(|error| format!("parse COTRA_WORKSPACES_JSON: {error}"))?;
        configs
            .into_iter()
            .map(|config| Workspace {
                id: config.id,
                root: config.root,
            })
            .collect()
    } else if let Some(root) = std::env::var_os("COTRA_WORKSPACE_ROOT") {
        vec![Workspace {
            id: std::env::var("COTRA_WORKSPACE_ID").unwrap_or_else(|_| "default".into()),
            root: PathBuf::from(root),
        }]
    } else {
        return Err(
            "configure COTRA_WORKSPACES_JSON or COTRA_WORKSPACE_ROOT before starting cotrad".into(),
        );
    };

    PolicyEngine::new(workspaces).map_err(|error| error.message)
}

fn handle_request(
    policy: &PolicyEngine,
    audit: &AuditLogger,
    approval: &impl ApprovalBroker,
    request: RequestEnvelope,
) -> ResponseEnvelope {
    let decision = match policy.authorize(&request) {
        Ok(decision) => decision,
        Err(error) => {
            let _ = audit.record(&request, POLICY_REVISION, "DENIED");
            return ResponseEnvelope::failure(request.request_id, error.code, error.message);
        }
    };

    match dispatch(policy, &decision.workspace, approval, &request) {
        Ok(value) => {
            if let Err(error) = audit.record(&request, decision.policy_revision, "SUCCESS") {
                return ResponseEnvelope::failure(
                    request.request_id,
                    FailureCode::InternalError,
                    format!("audit write failed: {error}"),
                );
            }
            ResponseEnvelope::success(&request, value, decision.policy_revision)
        }
        Err(error) => {
            let state = match error.code {
                FailureCode::ApprovalDenied => "APPROVAL_DENIED",
                FailureCode::ApprovalUnavailable => "APPROVAL_UNAVAILABLE",
                FailureCode::TargetStale => "TARGET_STALE",
                _ => "FAILED",
            };
            let _ = audit.record(&request, decision.policy_revision, state);
            ResponseEnvelope::failure(request.request_id, error.code, error.message)
        }
    }
}

fn dispatch(
    policy: &PolicyEngine,
    workspace: &Workspace,
    approval: &impl ApprovalBroker,
    request: &RequestEnvelope,
) -> Result<Value, ProviderError> {
    match (request.capability.as_str(), request.operation.as_str()) {
        ("system.status", "get") => Ok(json!({
            "name": "cotrad",
            "version": env!("CARGO_PKG_VERSION"),
            "internal_protocol_version": INTERNAL_PROTOCOL_VERSION,
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "pid": std::process::id(),
            "mode": "SG-000002_APPROVED_WRITE"
        })),
        ("workspace.get", "get") => {
            let configured = policy.workspace(&workspace.id).ok_or_else(|| {
                ProviderError::new(
                    FailureCode::WorkspaceDenied,
                    "workspace disappeared during request",
                )
            })?;
            Ok(json!({
                "id": configured.id,
                "root": configured.root,
                "policy_revision": POLICY_REVISION,
                "mode": "approved_write"
            }))
        }
        ("fs.stat", "stat") => fs_provider(workspace)?.stat(target(request)?),
        ("fs.list", "list") => fs_provider(workspace)?.list(target(request)?),
        ("fs.read", "read") => fs_provider(workspace)?.read_text(target(request)?),
        ("fs.search", "search") => {
            let query = request
                .arguments
                .get("query")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    ProviderError::new(
                        FailureCode::InvalidRequest,
                        "fs.search requires arguments.query",
                    )
                })?;
            let max_results = request
                .arguments
                .get("max_results")
                .and_then(Value::as_u64)
                .map(|value| value as usize);
            fs_provider(workspace)?.search_text(target(request)?, query, max_results)
        }
        ("fs.write", "preview") => {
            let content = content(request)?;
            Ok(fs_provider(workspace)?
                .preview_write(target(request)?, content)?
                .to_json())
        }
        ("fs.write", "write") => {
            let content = content(request)?;
            let relative = target(request)?;
            let provider = fs_provider(workspace)?;
            let preview = provider.preview_write(relative, content)?;
            let expected = request
                .arguments
                .get("expected_current_sha256")
                .and_then(Value::as_str);
            let create_if_missing = request
                .arguments
                .get("create_if_missing")
                .and_then(Value::as_bool)
                .unwrap_or(false);

            match (&preview.current_sha256, expected) {
                (Some(actual), Some(expected)) if actual == expected => {}
                (Some(_), Some(_)) => {
                    return Err(ProviderError::new(
                        FailureCode::TargetStale,
                        "target changed since preview",
                    ))
                }
                (Some(_), None) => {
                    return Err(ProviderError::new(
                        FailureCode::TargetStale,
                        "existing file requires expected_current_sha256 from preview",
                    ))
                }
                (None, Some(_)) => {
                    return Err(ProviderError::new(
                        FailureCode::TargetStale,
                        "target disappeared since preview",
                    ))
                }
                (None, None) if create_if_missing => {}
                (None, None) => {
                    return Err(ProviderError::new(
                        FailureCode::TargetStale,
                        "missing target requires create_if_missing=true",
                    ))
                }
            }

            let prompt = ApprovalPrompt {
                workspace_id: workspace.id.clone(),
                action: if preview.exists {
                    "overwrite UTF-8 file".into()
                } else {
                    "create UTF-8 file".into()
                },
                target: relative.to_owned(),
                summary: format!(
                    "{} bytes; new SHA-256 {}",
                    preview.bytes, preview.new_sha256
                ),
                digest: preview.approval_digest(),
            };

            match approval.request(&prompt) {
                Ok(ApprovalDecision::Approved) => {}
                Ok(ApprovalDecision::Denied) => {
                    return Err(ProviderError::new(
                        FailureCode::ApprovalDenied,
                        "local user denied file write",
                    ))
                }
                Err(error) => return Err(ProviderError::new(error.code, error.message)),
            }

            provider.write_text(relative, content, expected, create_if_missing)
        }
        _ => Err(ProviderError::new(
            FailureCode::CapabilityDenied,
            "dispatch reached an unauthorized capability",
        )),
    }
}

fn fs_provider(workspace: &Workspace) -> Result<FsProvider, ProviderError> {
    FsProvider::new(&workspace.root)
}

fn target(request: &RequestEnvelope) -> Result<&str, ProviderError> {
    request.target.as_deref().ok_or_else(|| {
        ProviderError::new(FailureCode::InvalidRequest, "filesystem target is required")
    })
}

fn content(request: &RequestEnvelope) -> Result<&str, ProviderError> {
    request
        .arguments
        .get("content")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ProviderError::new(
                FailureCode::InvalidRequest,
                "fs.write requires arguments.content",
            )
        })
}
