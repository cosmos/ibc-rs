use ibc_core::channel::types::acknowledgement::StatusValue;
use ibc_core::channel::types::error::ChannelError;
use ibc_core::handler::types::error::ContextError;

/// Defines the interchain account error type
#[derive(Clone, Debug)]
pub struct InterchainAccountError {
    /// Error code.
    pub code: Code,
    /// Refers to the type, value or place that error originated from or pertains to.
    pub origin: String,
    /// The given value that caused the error.
    pub given: Option<String>,
    /// The optional expected value.
    pub expected: Option<String>,
}

impl InterchainAccountError {
    /// Constructs a new interchain account error instance with the given code and message.
    pub fn new(code: Code, origin: impl Into<String>) -> Self {
        Self {
            code,
            origin: origin.into(),
            given: None,
            expected: None,
        }
    }

    /// Constructs an `Empty` error with the given message.
    pub fn empty(origin: impl Into<String>) -> Self {
        Self::new(Code::Empty, origin)
    }

    /// Constructs a `NotFound` error with the given message.
    pub fn not_found(origin: impl Into<String>) -> Self {
        Self::new(Code::NotFound, origin)
    }

    /// Constructs an `AlreadyExists` error with the given message.
    pub fn already_exists(origin: impl Into<String>) -> Self {
        Self::new(Code::AlreadyExists, origin)
    }

    /// Constructs an `Invalid` error with the given message.
    pub fn invalid(origin: impl Into<String>) -> Self {
        Self::new(Code::Invalid, origin)
    }

    /// Constructs a `MisMatch` error with the given message.
    pub fn mismatch(origin: impl Into<String>) -> Self {
        Self::new(Code::Mismatch, origin)
    }

    /// Constructs a `NotAllowed` error with the given message.
    pub fn not_allowed(origin: impl Into<String>) -> Self {
        Self::new(Code::NotAllowed, origin)
    }

    /// Constructs a `NotSupported` error with the given message.
    pub fn not_supported(origin: impl Into<String>) -> Self {
        Self::new(Code::NotSupported, origin)
    }

    /// Constructs a `Source` error with the given message.
    pub fn source(origin: impl ToString) -> Self {
        Self {
            code: Code::Source,
            origin: origin.to_string(),
            expected: None,
            given: None,
        }
    }

    /// Adds an expected value to the error message.
    pub fn expected(&self, expected: &impl ToString) -> Self {
        Self {
            expected: Some(expected.to_string()),
            ..self.clone()
        }
    }

    /// Adds a given value to the error message.
    pub fn given(&self, given: &impl ToString) -> Self {
        Self {
            given: Some(given.to_string()),
            ..self.clone()
        }
    }
}

impl core::fmt::Display for InterchainAccountError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Code: {} Origin: {}", self.code, self.origin)?;

        if let Some(given) = &self.given {
            write!(f, ", Given: {}", given)?;
        }

        if let Some(expected) = &self.expected {
            write!(f, ", Expected: {}", expected)?;
        }

        Ok(())
    }
}

impl From<ContextError> for InterchainAccountError {
    fn from(err: ContextError) -> InterchainAccountError {
        Self::source(err)
    }
}

impl From<ChannelError> for InterchainAccountError {
    fn from(err: ChannelError) -> InterchainAccountError {
        Self::source(err)
    }
}

impl From<InterchainAccountError> for StatusValue {
    fn from(err: InterchainAccountError) -> Self {
        StatusValue::new(err.to_string()).expect("error message cannot be empty")
    }
}

#[derive(Clone, Debug)]
pub enum Code {
    /// cannot be empty!
    Empty = 0,

    /// not found!
    NotFound = 1,

    /// already exists!
    AlreadyExists = 2,

    /// has invalid state!
    Invalid = 3,

    /// state mismatch!
    Mismatch = 4,

    /// not allowed!
    NotAllowed = 5,

    /// not supported!
    NotSupported = 6,

    /// from other source!
    Source = 7,
}

impl Code {
    /// Returns a string description of the error code.
    pub fn description(&self) -> &'static str {
        match self {
            Code::Empty => "cannot be empty!",
            Code::NotFound => "not found!",
            Code::AlreadyExists => "already exists!",
            Code::Invalid => "invalid state!",
            Code::Mismatch => "state mismatch!",
            Code::NotAllowed => "not allowed!",
            Code::NotSupported => "not supported!",
            Code::Source => "from other source!",
        }
    }
}

impl core::fmt::Display for Code {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self.description(), f)
    }
}
