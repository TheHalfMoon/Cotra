use cotra_contracts::FailureCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalPrompt {
    pub workspace_id: String,
    pub action: String,
    pub target: String,
    pub summary: String,
    pub digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalDecision {
    Approved,
    Denied,
}

#[derive(Debug, Clone)]
pub struct ApprovalError {
    pub code: FailureCode,
    pub message: String,
}

impl ApprovalError {
    fn unavailable(message: impl Into<String>) -> Self {
        Self {
            code: FailureCode::ApprovalUnavailable,
            message: message.into(),
        }
    }
}

pub trait ApprovalBroker {
    fn request(&self, prompt: &ApprovalPrompt) -> Result<ApprovalDecision, ApprovalError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LocalApprovalBroker;

impl ApprovalBroker for LocalApprovalBroker {
    fn request(&self, prompt: &ApprovalPrompt) -> Result<ApprovalDecision, ApprovalError> {
        platform_prompt(prompt)
    }
}

#[cfg(windows)]
fn platform_prompt(prompt: &ApprovalPrompt) -> Result<ApprovalDecision, ApprovalError> {
    use std::ffi::c_void;
    use std::ptr;

    type Hwnd = *mut c_void;
    const MB_YESNO: u32 = 0x0000_0004;
    const MB_ICONWARNING: u32 = 0x0000_0030;
    const MB_DEFBUTTON2: u32 = 0x0000_0100;
    const MB_SYSTEMMODAL: u32 = 0x0000_1000;
    const IDYES: i32 = 6;
    const IDNO: i32 = 7;

    #[link(name = "user32")]
    extern "system" {
        fn MessageBoxW(hwnd: Hwnd, text: *const u16, caption: *const u16, kind: u32) -> i32;
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    let body = format!(
        "Cotra requests a local action.\n\nWorkspace: {}\nAction: {}\nTarget: {}\n{}\nDigest: {}\n\nApprove this exact operation?",
        prompt.workspace_id, prompt.action, prompt.target, prompt.summary, prompt.digest
    );
    let body = wide(&body);
    let caption = wide("Cotra approval");
    let result = unsafe {
        MessageBoxW(
            ptr::null_mut(),
            body.as_ptr(),
            caption.as_ptr(),
            MB_YESNO | MB_ICONWARNING | MB_DEFBUTTON2 | MB_SYSTEMMODAL,
        )
    };

    match result {
        IDYES => Ok(ApprovalDecision::Approved),
        IDNO => Ok(ApprovalDecision::Denied),
        _ => Err(ApprovalError::unavailable(format!(
            "Windows approval prompt returned unexpected result {result}"
        ))),
    }
}

#[cfg(not(windows))]
fn platform_prompt(_prompt: &ApprovalPrompt) -> Result<ApprovalDecision, ApprovalError> {
    Err(ApprovalError::unavailable(
        "local approval UI is Windows-only in the current Cotra runtime",
    ))
}

#[cfg(test)]
pub mod test_support {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    pub struct FixedApprovalBroker(pub ApprovalDecision);

    impl ApprovalBroker for FixedApprovalBroker {
        fn request(&self, _prompt: &ApprovalPrompt) -> Result<ApprovalDecision, ApprovalError> {
            Ok(self.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::FixedApprovalBroker;
    use super::*;

    fn prompt() -> ApprovalPrompt {
        ApprovalPrompt {
            workspace_id: "default".into(),
            action: "fs.write".into(),
            target: "notes.txt".into(),
            summary: "12 UTF-8 bytes".into(),
            digest: "abc".into(),
        }
    }

    #[test]
    fn fixed_broker_can_deny() {
        let broker = FixedApprovalBroker(ApprovalDecision::Denied);
        assert_eq!(broker.request(&prompt()).unwrap(), ApprovalDecision::Denied);
    }

    #[test]
    fn fixed_broker_can_approve() {
        let broker = FixedApprovalBroker(ApprovalDecision::Approved);
        assert_eq!(
            broker.request(&prompt()).unwrap(),
            ApprovalDecision::Approved
        );
    }
}
