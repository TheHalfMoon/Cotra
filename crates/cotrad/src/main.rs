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

    eprintln!(
        "cotrad ready: protocol={} audit={}",
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
            Ok(request) => handle_request(&policy, &audit, request),
            Err(error) => ResponseEnvelope::failure(
                "unknown",
                FailureCode::InvalidRequest,
                format!("invalid request JSON: {error}"),
            ),
        };

        serde_json::to_writer(&mut stdout, &response)
            .map_err(|error| format!("serialize response: {error}"))?;
        stdout.write_all(b"\n")
            .map_err(|error| format!("write response: {error}"))?;
        stdout.flush()
            .map_err(|error| format!("flush response: {error}"))?;
    }

    Ok(())
}

fn load_policy() -> Result<PolicyEngine, String> {
    let workspaces = if let Ok(raw) = std::env::var("COTRA_WORKSPACES_JSON") {
        let configs: Vec<WorkspaceConfig> = serde_json::from_str(&raw)
            .map_err(|error| format!("parse COTRA_WORKSPACES_JSON: {error}"))?;
        configs.into_iter()
            .map(|config| Workspace { id: config.id, root: config.root })
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
    request: RequestEnvelope,
) -> ResponseEnvelope {
    let decision = match policy.authorize(&request) {
        Ok(decision) => decision,
        Err(error) => {
            let _ = audit.record(&request, POLICY_REVISION, "DENIED");
            return ResponseEnvelope::failure(request.request_id, error.code, error.message);
        }
    };

    match dispatch(policy, &decision.workspace, &request) {
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
            let _ = audit.record(&request, decision.policy_revision, "FAILED");
            ResponseEnvelope::failure(request.request_id, error.code, error.message)
        }
    }
}

fn dispatch(
    policy: &PolicyEngine,
    workspace: &Workspace,
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
            "mode": "SG-000001_READ_ONLY"
        })),
        ("workspace.get", "get") => {
            let configured = policy.workspace(&workspace.id).ok_or_else(|| ProviderError {
                code: FailureCode::WorkspaceDenied,
                message: "workspace disappeared during request".into(),
            })?;
            Ok(json!({
                "id": configured.id,
                "root": configured.root,
                "policy_revision": POLICY_REVISION,
                "mode": "read_only"
            }))
        }
        ("fs.stat", "stat") => fs_provider(workspace)?.stat(target(request)?),
        ("fs.list", "list") => fs_provider(workspace)?.list(target(request)?),
        ("fs.read", "read") => fs_provider(workspace)?.read_text(target(request)?),
        ("fs.search", "search") => {
            let query = request.arguments.get("query")
                .and_then(Value::as_str)
                .ok_or_else(|| ProviderError {
                    code: FailureCode::InvalidRequest,
                    message: "fs.search requires arguments.query".into(),
                })?;
            let max_results = request.arguments.get("max_results")
                .and_then(Value::as_u64)
                .map(|value| value as usize);
            fs_provider(workspace)?.search_text(target(request)?, query, max_results)
        }
        _ => Err(ProviderError {
            code: FailureCode::CapabilityDenied,
            message: "dispatch reached an unauthorized capability".into(),
        }),
    }
}

fn fs_provider(workspace: &Workspace) -> Result<FsProvider, ProviderError> {
    FsProvider::new(&workspace.root)
}

fn target(request: &RequestEnvelope) -> Result<&str, ProviderError> {
    request.target.as_deref().ok_or_else(|| ProviderError {
        code: FailureCode::InvalidRequest,
        message: "filesystem target is required".into(),
    })
}
