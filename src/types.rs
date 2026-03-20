use serde::{Deserialize, Serialize};

/// Docker Hub image logo API response
#[derive(Debug, Clone, Deserialize)]
pub struct DockerHubLogoResponse {
    pub logo_url: Option<String>,
    pub last_updated: Option<String>,
}

/// Docker Hub organization info response
#[derive(Debug, Clone, Deserialize)]
pub struct DockerHubOrgResponse {
    pub id: String,
    pub orgname: Option<String>,
    pub full_name: Option<String>,
    pub location: Option<String>,
    pub company: Option<String>,
    pub gravatar_url: Option<String>,
    pub gravatar_email: Option<String>,
}

/// Docker Hub user info response
#[derive(Debug, Clone, Deserialize)]
pub struct DockerHubUserResponse {
    pub id: String,
    pub username: Option<String>,
    pub full_name: Option<String>,
    pub gravatar_url: Option<String>,
    pub gravatar_email: Option<String>,
}

/// Image icon source - Identify the source of the icon
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IconSource {
    /// Logo from Docker Hub image
    DockerHubLogo { url: String },
    /// Gravatar from Docker Hub organization
    DockerHubOrgGravatar { url: String },
    /// Logo from Docker Official Image (via GitHub docker-library/docs)
    DockerOfficialImage { url: String },
    /// Icon from devicons/devicon (via jsDelivr CDN)
    Devicon { url: String },
    /// Avatar from GitHub Container Registry
    GhcrAvatar { url: String },
    /// User-defined custom icon URL
    Custom { url: String },
    /// Icon not found
    NotFound,
}

impl IconSource {
    /// Get the URL of the icon, or None if not found
    pub fn url(&self) -> Option<&str> {
        match self {
            IconSource::DockerHubLogo { url } => Some(url),
            IconSource::DockerHubOrgGravatar { url } => Some(url),
            IconSource::DockerOfficialImage { url } => Some(url),
            IconSource::Devicon { url } => Some(url),
            IconSource::GhcrAvatar { url } => Some(url),
            IconSource::Custom { url } => Some(url),
            IconSource::NotFound => None,
        }
    }

    /// Check if the icon was found
    pub fn is_found(&self) -> bool {
        !matches!(self, IconSource::NotFound)
    }

    /// Create a custom icon source
    pub fn custom(url: impl Into<String>) -> Self {
        IconSource::Custom { url: url.into() }
    }
}

impl Default for IconSource {
    fn default() -> Self {
        IconSource::NotFound
    }
}
