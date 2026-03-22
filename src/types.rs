use serde::{Deserialize, Serialize};

/// Docker Hub image logo API response
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DockerHubLogoResponse {
    pub logo_url: Option<String>,
}

/// Docker Hub organization info response
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DockerHubOrgResponse {
    pub gravatar_url: Option<String>,
}

/// Image icon source
///
/// Identifies where the icon was resolved from. Every variant carries
/// the URL of the icon.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum IconSource {
    /// Logo from Docker Hub image
    DockerHubLogo { url: String },
    /// Gravatar from Docker Hub organization
    DockerHubOrgGravatar { url: String },
    /// Logo from Docker Official Image (via GitHub docker-library/docs)
    DockerOfficialImage { url: String },
    /// Icon from devicons/devicon (via jsDelivr CDN)
    #[cfg(feature = "devicon")]
    Devicon { url: String },
    /// Avatar from GitHub Container Registry
    GhcrAvatar { url: String },
    /// User-defined custom icon URL
    Custom { url: String },
}

impl IconSource {
    /// Get the URL of the icon
    pub fn url(&self) -> &str {
        match self {
            Self::DockerHubLogo { url }
            | Self::DockerHubOrgGravatar { url }
            | Self::DockerOfficialImage { url }
            | Self::GhcrAvatar { url }
            | Self::Custom { url } => url,
            #[cfg(feature = "devicon")]
            Self::Devicon { url } => url,
        }
    }

    /// Create a custom icon source
    pub fn custom(url: impl Into<String>) -> Self {
        Self::Custom { url: url.into() }
    }
}

/// A resolved icon with its source metadata and raw image data
#[derive(Debug, Clone)]
pub struct Icon {
    source: IconSource,
    data: Vec<u8>,
    content_type: Option<String>,
}

impl Icon {
    pub(crate) fn new(source: IconSource, data: Vec<u8>, content_type: Option<String>) -> Self {
        Self { source, data, content_type }
    }

    /// The source this icon was resolved from
    pub fn source(&self) -> &IconSource {
        &self.source
    }

    /// The URL of the icon
    pub fn url(&self) -> &str {
        self.source.url()
    }

    /// Raw image bytes
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Consume the icon and return the raw image bytes
    pub fn into_data(self) -> Vec<u8> {
        self.data
    }

    /// MIME content type, if known (e.g. `image/png`, `image/svg+xml`)
    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url() {
        let source = IconSource::DockerHubLogo {
            url: "https://example.com/logo.png".to_string(),
        };
        assert_eq!(source.url(), "https://example.com/logo.png");
    }

    #[test]
    #[cfg(feature = "devicon")]
    fn test_devicon() {
        let icon = IconSource::Devicon {
            url: "https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/nginx/nginx-original.svg".to_string(),
        };
        assert_eq!(
            icon.url(),
            "https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/nginx/nginx-original.svg"
        );
    }

    #[test]
    fn test_custom() {
        let icon = IconSource::custom("https://example.com/icon.png");
        assert_eq!(icon.url(), "https://example.com/icon.png");
    }
}
