use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub type_url: String,
    pub title: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
}

impl ProblemDetails {
    pub fn bad_request(detail: impl Into<String>) -> Self {
        Self {
            type_url: "https://ofscp.dev/problems/bad-request".to_string(),
            title: "Bad Request".to_string(),
            status: 400,
            detail: Some(detail.into()),
            instance: None,
        }
    }

    pub fn unauthorized(detail: impl Into<String>) -> Self {
        Self {
            type_url: "https://ofscp.dev/problems/unauthorized".to_string(),
            title: "Unauthorized".to_string(),
            status: 401,
            detail: Some(detail.into()),
            instance: None,
        }
    }

    pub fn not_found(detail: impl Into<String>) -> Self {
        Self {
            type_url: "https://ofscp.dev/problems/not-found".to_string(),
            title: "Not Found".to_string(),
            status: 404,
            detail: Some(detail.into()),
            instance: None,
        }
    }

    pub fn conflict(detail: impl Into<String>) -> Self {
        Self {
            type_url: "https://ofscp.dev/problems/conflict".to_string(),
            title: "Conflict".to_string(),
            status: 409,
            detail: Some(detail.into()),
            instance: None,
        }
    }

    pub fn internal_error(detail: impl Into<String>) -> Self {
        Self {
            type_url: "https://ofscp.dev/problems/internal-error".to_string(),
            title: "Internal Server Error".to_string(),
            status: 500,
            detail: Some(detail.into()),
            instance: None,
        }
    }
}

pub fn try_problem_detail(body: &str) -> Option<String> {
    let parsed = serde_json::from_str::<ProblemDetails>(body).ok()?;
    if let Some(detail) = parsed.detail {
        if !detail.trim().is_empty() {
            return Some(detail);
        }
    }
    if !parsed.title.trim().is_empty() {
        return Some(parsed.title);
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    Network(String),
    Http { status: u16, body: String },
    Deserialize(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Network(msg) => write!(f, "Network error: {}", msg),
            ApiError::Http { status, body } => write!(f, "HTTP {}: {}", status, body),
            ApiError::Deserialize(msg) => write!(f, "Deserialization error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}
