//! A refusal, as the client is told it: a stable code to act on and a sentence to show.

/// Why a request was not done. Nothing was changed by a request that answers one of these.
#[derive(Debug, thiserror::Error)]
pub enum SocialError {
    /// The request is malformed or breaks a rule its sender can see (a body too long, an unknown method).
    #[error("{0}")]
    BadRequest(String),
    /// What it names does not exist, or is not the caller's to see; the two are not told apart.
    #[error("{0}")]
    NotFound(String),
    /// The caller may see it but may not do this.
    #[error("{0}")]
    Forbidden(String),
    /// It conflicts with what is already so.
    #[error("{0}")]
    Conflict(String),
    /// Too much, too fast; nothing was done.
    #[error("{0}")]
    RateLimited(String),
    /// The account is still on a name QOR ID made for it (ADR-091 decision 5): it chooses its own first.
    #[error("choose your QOR ID name before you post")]
    NameNotChosen,
    /// The database failed. Never says why, which may hold a statement's values.
    #[error("Social could not do that just now; nothing was changed")]
    Database(#[from] sqlx::Error),
    /// Something of Social's own failed.
    #[error("Social could not do that just now; nothing was changed")]
    Internal(#[from] anyhow::Error),
}

impl SocialError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "bad_request",
            Self::NotFound(_) => "not_found",
            Self::Forbidden(_) => "forbidden",
            Self::Conflict(_) => "conflict",
            Self::RateLimited(_) => "rate_limited",
            Self::NameNotChosen => "name_not_chosen",
            Self::Database(_) | Self::Internal(_) => "unavailable",
        }
    }

    pub fn bad(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden(message.into())
    }
}

pub type SocialResult<T> = Result<T, SocialError>;
