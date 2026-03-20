use reqwest::{Client, StatusCode};
use tracing::{debug, warn};

use crate::error::{Error, Result};
use crate::parser::ImageReference;
use crate::types::{DockerHubLogoResponse, DockerHubOrgResponse, IconSource};

/// Mirror icon service
///
/// Used to fetch Docker image icons from various sources.
///
/// # Supported sources
///
/// - Docker Hub image logos
/// - Docker Hub organization gravatars
/// - devicons/devicon (via jsDelivr CDN)
/// - Docker Official Images (via jsDelivr CDN)
/// - GitHub Container Registry (via GitHub Avatar)
///
/// # Example
///
/// ```no_run
/// use dimicon::IconService;
///
/// #[tokio::main]
/// async fn main() {
///     let service = IconService::new();
///
///     // Get the icon for nginx
///     let icon = service.get_icon("nginx").await.unwrap();
///     if let Some(url) = icon.url() {
///         println!("Icon URL: {}", url);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct IconService {
    client: Client,
}

impl Default for IconService {
    fn default() -> Self {
        Self::new()
    }
}

impl IconService {
    /// Create a new icon service instance
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("dimicon/0.1.0")
            .build()
            .expect("Failed to build HTTP client");

        Self { client }
    }

    /// Create a new icon service instance with a custom HTTP client
    pub fn with_client(client: Client) -> Self {
        Self { client }
    }

    /// Get the icon for an image
    ///
    /// Tries multiple sources in order of priority:
    /// 1. Registry-specific free sources (Docker Official Image logo, GHCR avatar)
    /// 2. devicons/devicon via jsDelivr CDN (universal, works for any registry)
    /// 3. Rate-limited Docker Hub APIs (org Gravatar, image logo)
    ///
    /// # Arguments
    ///
    /// * `image` - Image reference string, such as "nginx", "myuser/myapp:latest", "ghcr.io/owner/app"
    ///
    /// # Returns
    ///
    /// Returns an `IconSource` containing the icon URL or `NotFound`
    pub async fn get_icon(&self, image: &str) -> Result<IconSource> {
        let parsed = ImageReference::parse(image)?;
        self.get_icon_for_ref(&parsed).await
    }

    /// Get the icon for a parsed image reference
    pub async fn get_icon_for_ref(&self, parsed: &ImageReference) -> Result<IconSource> {
        // 1. Try registry-specific free sources first
        if parsed.is_docker_hub() {
            if parsed.is_docker_official() {
                if let Some(icon) = self.get_docker_official_image_logo(&parsed.name).await? {
                    return Ok(icon);
                }
            }
        } else if parsed.is_ghcr() {
            let result = self.get_ghcr_icon(parsed).await?;
            if result.is_found() {
                return Ok(result);
            }
        }

        // 2. Try devicons/devicon via jsDelivr CDN (universal, works for any registry)
        if let Some(icon) = self.get_devicon_logo(&parsed.name).await? {
            return Ok(icon);
        }

        // 3. Try rate-limited Docker Hub APIs as last resort
        if parsed.is_docker_hub() {
            let repo_name = parsed.docker_hub_repo_name();
            if let Some(icon) = self.get_docker_hub_repo_logo(&repo_name).await? {
                return Ok(icon);
            }

            if let Some(icon) = self.get_docker_hub_org_gravatar(&parsed.namespace).await? {
                return Ok(icon);
            }
        }

        Ok(IconSource::NotFound)
    }

    /// Get the Docker Hub image logo
    ///
    /// API: `https://hub.docker.com/api/media/repos_logo/v1/{namespace}%2F{image}?type=logo`
    async fn get_docker_hub_repo_logo(&self, repo_name: &str) -> Result<Option<IconSource>> {
        let encoded_repo = urlencoding::encode(repo_name);
        let url = format!(
            "https://hub.docker.com/api/media/repos_logo/v1/{}?type=logo",
            encoded_repo
        );

        debug!("Fetching Docker Hub logo from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let logo_response: DockerHubLogoResponse = response.json().await?;
                if let Some(logo_url) = logo_response.logo_url {
                    return Ok(Some(IconSource::DockerHubLogo { url: logo_url }));
                }
                Ok(None)
            }
            StatusCode::NOT_FOUND => {
                debug!("No logo found for Docker Hub repo: {}", repo_name);
                Ok(None)
            }
            StatusCode::TOO_MANY_REQUESTS => {
                warn!("Rate limited by Docker Hub API");
                Err(Error::RateLimited)
            }
            status => {
                debug!("Docker Hub logo API returned status: {}", status);
                Ok(None)
            }
        }
    }

    /// Get the Docker Hub organization's Gravatar
    ///
    /// API: `https://hub.docker.com/v2/orgs/{namespace}/`
    async fn get_docker_hub_org_gravatar(&self, namespace: &str) -> Result<Option<IconSource>> {
        if namespace == "library" {
            // library is Docker Official Images, no need to get Gravatar
            return Ok(None);
        }

        let url = format!("https://hub.docker.com/v2/orgs/{}/", namespace);
        debug!("Fetching Docker Hub org info from: {}", url);

        let response = self.client.get(&url).send().await?;

        match response.status() {
            StatusCode::OK => {
                let org_response: DockerHubOrgResponse = response.json().await?;
                if let Some(gravatar_url) = org_response.gravatar_url {
                    if !gravatar_url.is_empty() {
                        return Ok(Some(IconSource::DockerHubOrgGravatar { url: gravatar_url }));
                    }
                }
                Ok(None)
            }
            StatusCode::PERMANENT_REDIRECT | StatusCode::TEMPORARY_REDIRECT => {
                // May redirect to user API, ignore for now
                debug!("Docker Hub org API redirected for: {}", namespace);
                Ok(None)
            }
            StatusCode::NOT_FOUND => {
                debug!("No org found for: {}", namespace);
                Ok(None)
            }
            StatusCode::TOO_MANY_REQUESTS => {
                warn!("Rate limited by Docker Hub API");
                Err(Error::RateLimited)
            }
            status => {
                debug!("Docker Hub org API returned status: {}", status);
                Ok(None)
            }
        }
    }

    /// Get the icon from devicons/devicon via jsDelivr CDN
    ///
    /// URL: `https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/{name}/{name}-original.svg`
    async fn get_devicon_logo(&self, image_name: &str) -> Result<Option<IconSource>> {
        let url = format!(
            "https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/{name}/{name}-original.svg",
            name = image_name
        );

        debug!("Checking devicon logo at: {}", url);

        let response = self.client.head(&url).send().await?;

        if response.status().is_success() {
            Ok(Some(IconSource::Devicon { url }))
        } else {
            debug!("Devicon logo not found for: {}", image_name);
            Ok(None)
        }
    }

    /// Get the Docker Official Image logo
    ///
    /// Fetches the logo from the GitHub docker-library/docs repository via jsDelivr
    /// URL: `https://cdn.jsdelivr.net/gh/docker-library/docs@master/{image}/logo.png`
    async fn get_docker_official_image_logo(
        &self,
        image_name: &str,
    ) -> Result<Option<IconSource>> {
        let url = format!(
            "https://cdn.jsdelivr.net/gh/docker-library/docs@master/{}/logo.png",
            image_name
        );

        debug!("Checking Docker Official Image logo at: {}", url);

        // Use HEAD request to check if the resource exists
        let response = self.client.head(&url).send().await?;

        if response.status().is_success() {
            Ok(Some(IconSource::DockerOfficialImage { url }))
        } else {
            debug!("Docker Official Image logo not found for: {}", image_name);
            Ok(None)
        }
    }

    /// Get the GitHub Container Registry image icon
    ///
    /// Uses the GitHub Avatar API
    async fn get_ghcr_icon(&self, parsed: &ImageReference) -> Result<IconSource> {
        // The namespace of ghcr.io is the GitHub username or organization name
        let avatar_url = format!("https://avatars.githubusercontent.com/{}", parsed.namespace);

        debug!("Using GitHub avatar for ghcr.io: {}", avatar_url);

        // Check if the avatar exists
        let response = self.client.head(&avatar_url).send().await?;

        if response.status().is_success() {
            Ok(IconSource::GhcrAvatar { url: avatar_url })
        } else {
            debug!("GitHub avatar not found for: {}", parsed.namespace);
            Ok(IconSource::NotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_source_url() {
        let source = IconSource::DockerHubLogo {
            url: "https://example.com/logo.png".to_string(),
        };
        assert_eq!(source.url(), Some("https://example.com/logo.png"));

        let not_found = IconSource::NotFound;
        assert_eq!(not_found.url(), None);
    }

    #[test]
    fn test_icon_source_is_found() {
        assert!(IconSource::DockerHubLogo {
            url: "test".to_string()
        }
        .is_found());
        assert!(!IconSource::NotFound.is_found());
    }

    #[test]
    fn test_icon_source_devicon() {
        let icon = IconSource::Devicon {
            url: "https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/nginx/nginx-original.svg".to_string(),
        };
        assert_eq!(
            icon.url(),
            Some("https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/nginx/nginx-original.svg")
        );
        assert!(icon.is_found());
    }

    #[test]
    fn test_icon_source_custom() {
        let icon = IconSource::custom("https://example.com/icon.png");
        assert_eq!(icon.url(), Some("https://example.com/icon.png"));
    }
}
