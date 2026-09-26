use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const INTERNAL_PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestEnvelope {
    pub version: u32,
    pub request_id: String,
    pub client_session_id: String,
    pub workspace_id: String,
    pub capability: String,
    pub operation: String,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FailureCode {
    InvalidRequest,
    CapabilityDenied,
    WorkspaceDenied,
    PathEscape,
    PathRaceDetected,
    ApprovalRequired,
    ApprovalDenied,
    ApprovalUnavailable,
    TargetStale,
    ProviderUnavailable,
    ProcessTimeout,
    ProcessTerminationUnverified,
    OutputLimit,
    PostconditionFailed,
    InternalError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    pub code: FailureCode,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub workspace_id: String,
    pub policy_revision: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseEnvelope {
    pub version: u32,
    pub request_id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorBody>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<Evidence>,
}

impl ResponseEnvelope {
    pub fn success(request: &RequestEnvelope, result: Value, policy_revision: &str) -> Self {
        Self {
            version: INTERNAL_PROTOCOL_VERSION,
            request_id: request.request_id.clone(),
            ok: true,
            result: Some(result),
            error: None,
            evidence: Some(Evidence {
                workspace_id: request.workspace_id.clone(),
                policy_revision: policy_revision.to_owned(),
                target: request.target.clone(),
            }),
        }
    }

    pub fn failure(
        request_id: impl Into<String>,
        code: FailureCode,
        message: impl Into<String>,
    ) -> Self {
        Self {
            version: INTERNAL_PROTOCOL_VERSION,
            request_id: request_id.into(),
            ok: false,
            result: None,
            error: Some(ErrorBody {
                code,
                message: message.into(),
            }),
            evidence: None,
        }
    }
}
