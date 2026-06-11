//! exception-handler — platform exception / signal handler registration
use std::fmt;

#[derive(Debug, Clone)]
pub struct ExceptionInfo {
    pub signal: i32,
    pub address: usize,
    pub fault_type: FaultType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FaultType {
    Read,
    Write,
    Execute,
    Unknown,
}

#[derive(Debug)]
pub enum HandlerError {
    AlreadyInstalled,
    PlatformUnsupported,
}

impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HandlerError::AlreadyInstalled => write!(f, "handler already installed"),
            HandlerError::PlatformUnsupported => write!(f, "platform not supported"),
        }
    }
}

impl std::error::Error for HandlerError {}

pub type ExceptionCallback = Box<dyn Fn(&ExceptionInfo) -> bool + Send + Sync>;

pub struct ExceptionHandler {
    #[allow(dead_code)]
    callback: Option<ExceptionCallback>,
}

impl ExceptionHandler {
    pub fn new() -> Self { Self { callback: None } }
    pub fn install(&mut self, _cb: ExceptionCallback) -> Result<(), HandlerError> {
        Err(HandlerError::PlatformUnsupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_handler() {
        let h = ExceptionHandler::new();
        assert!(h.callback.is_none());
    }
}
