use cotra_contracts::{FailureCode, RequestEnvelope};
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

pub const POLICY_REVISION: &str = "sg-000001-v1";

#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: String,
    pub root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PolicyDecision {
    pub workspace: Workspace,
    pub policy_revision: &'static str,
}

#[derive(Debug, Clone)]
pub struct PolicyError {
    pub code: FailureCode,
    pub message: String,
}

impl PolicyError {
    fn new(code: FailureCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PolicyEngine {
    workspaces: HashMap<String, Workspace>,
}

impl PolicyEngine {
    pub fn new(workspaces: Vec<Workspace>) -> Result<Self, PolicyError> {
        if workspaces.is_empty() {
            return Err(PolicyError::new(
                FailureCode::WorkspaceDenied,
                "at least one workspace must be configured",
            ));
        }

        let mut map = HashMap::new();
        for mut workspace in workspaces {
            if workspace.id.trim().is_empty() {
                return Err(PolicyError::new(
                    FailureCode::InvalidRequest,
                    "workspace id cannot be empty",
                ));
            }
            workspace.root = std::fs::canonicalize(&workspace.root).map_err(|error| {
                PolicyError::new(
                    FailureCode::WorkspaceDenied,
                    format!(
                        "workspace root could not be resolved for {}: {error}",
                        workspace.id
                    ),
                )
            })?;
            if !workspace.root.is_dir() {
                return Err(PolicyError::new(
                    FailureCode::WorkspaceDenied,
                    format!("workspace root is not a directory: {}", workspace.id),
                ));
            }
            if map.insert(workspace.id.clone(), workspace).is_some() {
                return Err(PolicyError::new(
                    FailureCode::InvalidRequest,
                    "duplicate workspace id",
                ));
            }
        }

        Ok(Self { workspaces: map })
    }

    pub fn workspace(&self, id: &str) -> Option<&Workspace> {
        self.workspaces.get(id)
    }

    pub fn authorize(&self, request: &RequestEnvelope) -> Result<PolicyDecision, PolicyError> {
        if request.version != cotra_contracts::INTERNAL_PROTOCOL_VERSION {
            return Err(PolicyError::new(
                FailureCode::InvalidRequest,
                "unsupported internal protocol version",
            ));
        }

        let workspace = self.workspaces.get(&request.workspace_id).ok_or_else(|| {
            PolicyError::new(
                FailureCode::WorkspaceDenied,
                "requested workspace is not configured",
            )
        })?;

        let allowed = matches!(
            (request.capability.as_str(), request.operation.as_str()),
            ("system.status", "get")
                | ("workspace.get", "get")
                | ("fs.stat", "stat")
                | ("fs.list", "list")
                | ("fs.read", "read")
                | ("fs.search", "search")
        );

        if !allowed {
            return Err(PolicyError::new(
                FailureCode::CapabilityDenied,
                format!(
                    "capability/operation is not allowed in SG-000001: {}/{}",
                    request.capability, request.operation
                ),
            ));
        }

        if request.capability.starts_with("fs.") {
            let target = request.target.as_deref().ok_or_else(|| {
                PolicyError::new(FailureCode::InvalidRequest, "filesystem target is required")
            })?;
            validate_relative_target(target)?;
        }

        Ok(PolicyDecision {
            workspace: workspace.clone(),
            policy_revision: POLICY_REVISION,
        })
    }
}

pub fn validate_relative_target(target: &str) -> Result<(), PolicyError> {
    if target.contains('\0') {
        return Err(PolicyError::new(
            FailureCode::InvalidRequest,
            "path contains a NUL byte",
        ));
    }

    let portable = target.replace('/', "\\");
    let bytes = portable.as_bytes();
    let drive_prefixed = bytes.len() >= 2 && bytes[1] == b':';
    let device_prefixed = portable.starts_with("\\\\")
        || portable.starts_with("\\?\\")
        || portable.starts_with("\\.\\");
    if drive_prefixed || device_prefixed || target.starts_with('/') {
        return Err(PolicyError::new(
            FailureCode::PathEscape,
            "absolute, UNC, or device paths are not allowed",
        ));
    }

    let path = Path::new(target);
    if path.is_absolute() {
        return Err(PolicyError::new(
            FailureCode::PathEscape,
            "absolute paths are not allowed",
        ));
    }

    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(PolicyError::new(
                    FailureCode::PathEscape,
                    "parent/root/prefix path components are not allowed",
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cotra_contracts::RequestEnvelope;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("cotra-policy-{suffix}"));
        std::fs::create_dir_all(&root).expect("create temp workspace");
        root
    }

    fn request(target: &str) -> RequestEnvelope {
        RequestEnvelope {
            version: 1,
            request_id: "r1".into(),
            client_session_id: "s1".into(),
            workspace_id: "default".into(),
            capability: "fs.read".into(),
            operation: "read".into(),
            target: Some(target.into()),
            arguments: json!({}),
        }
    }

    #[test]
    fn rejects_parent_escape() {
        let root = temp_root();
        let engine = PolicyEngine::new(vec![Workspace {
            id: "default".into(),
            root: root.clone(),
        }])
        .expect("policy");

        let error = engine
            .authorize(&request("../outside.txt"))
            .expect_err("escape must fail");
        assert_eq!(error.code, FailureCode::PathEscape);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_windows_drive_path_portably() {
        assert!(validate_relative_target(r"C:\Windows\System32").is_err());
        assert!(validate_relative_target(r"\server\share\file").is_err());
        assert!(validate_relative_target(r"\?\C:\Windows").is_err());
    }

    #[test]
    fn denies_unknown_capability() {
        let root = temp_root();
        let engine = PolicyEngine::new(vec![Workspace {
            id: "default".into(),
            root: root.clone(),
        }])
        .expect("policy");
        let mut req = request("file.txt");
        req.capability = "process.spawn".into();
        req.operation = "spawn".into();
        let error = engine.authorize(&req).expect_err("must be denied");
        assert_eq!(error.code, FailureCode::CapabilityDenied);
        let _ = std::fs::remove_dir_all(root);
    }
}
