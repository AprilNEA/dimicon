use reqwest::{Client, StatusCode};
use tracing::{debug, warn};

use crate::error::{Error, Result};
use crate::parser::ImageReference;
use crate::types::{DockerHubLogoResponse, DockerHubOrgResponse, IconSource};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Docker image icon service
///
/// Fetches Docker image icons from various sources including Docker Hub,
/// GitHub Container Registry, and devicons.
///
/// # Example
///
/// ```no_run
/// use dimicon::IconService;
///
/// #[tokio::main]
/// async fn main() {
///     let service = IconService::new();
///     if let Some(icon) = service.get_icon("nginx").await.unwrap() {
///         println!("Icon URL: {}", icon.url());
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
        Self {
            client: Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    /// Create a new icon service instance with a custom HTTP client
    pub fn with_client(client: Client) -> Self {
        Self { client }
    }

    /// Get the icon for an image
    ///
    /// Tries multiple sources in order of priority:
    /// 1. Registry-specific free sources (Docker Official Image logo, GHCR avatar)
    /// 2. devicons/devicon via jsDelivr CDN (requires `devicon` feature)
    /// 3. Rate-limited Docker Hub APIs (org Gravatar, image logo)
    ///
    /// Returns `Ok(None)` if no icon could be found.
    pub async fn get_icon(&self, image: &str) -> Result<Option<IconSource>> {
        let parsed = ImageReference::parse(image)?;
        self.get_icon_for_ref(&parsed).await
    }

    /// Get the icon for a parsed image reference
    pub async fn get_icon_for_ref(
        &self,
        parsed: &ImageReference,
    ) -> Result<Option<IconSource>> {
        // 1. Registry-specific free sources
        if parsed.is_docker_hub() {
            if parsed.is_docker_official() {
                if let Some(icon) = self.fetch_docker_official_logo(parsed.name()).await? {
                    return Ok(Some(icon));
                }
            }
        } else if parsed.is_ghcr() {
            if let Some(icon) = self.fetch_ghcr_avatar(parsed.namespace()).await? {
                return Ok(Some(icon));
            }
        }

        // 2. devicons/devicon via jsDelivr CDN
        #[cfg(feature = "devicon")]
        if let Some(icon) = self.fetch_devicon_logo(parsed.name()).await? {
            return Ok(Some(icon));
        }

        // 3. Rate-limited Docker Hub APIs as last resort
        if parsed.is_docker_hub() {
            let repo_name = parsed.docker_hub_repo_name();
            if let Some(icon) = self.fetch_docker_hub_logo(&repo_name).await? {
                return Ok(Some(icon));
            }
            if let Some(icon) = self.fetch_docker_hub_org_gravatar(parsed.namespace()).await? {
                return Ok(Some(icon));
            }
        }

        Ok(None)
    }

    /// Fetch logo via the Docker Hub media API
    async fn fetch_docker_hub_logo(&self, repo_name: &str) -> Result<Option<IconSource>> {
        let encoded = urlencoding::encode(repo_name);
        let url = format!(
            "https://hub.docker.com/api/media/repos_logo/v1/{encoded}?type=logo"
        );

        debug!("Fetching Docker Hub logo: {url}");

        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        match resp.status() {
            StatusCode::OK => {
                let body: DockerHubLogoResponse = resp.json().await?;
                Ok(body.logo_url.map(|url| IconSource::DockerHubLogo { url }))
            }
            StatusCode::TOO_MANY_REQUESTS => {
                warn!("Rate limited by Docker Hub API");
                Err(Error::RateLimited)
            }
            status => {
                debug!("Docker Hub logo API returned {status} for {repo_name}");
                Ok(None)
            }
        }
    }

    /// Fetch organization Gravatar from Docker Hub
    async fn fetch_docker_hub_org_gravatar(
        &self,
        namespace: &str,
    ) -> Result<Option<IconSource>> {
        if namespace == "library" {
            return Ok(None);
        }

        let url = format!("https://hub.docker.com/v2/orgs/{namespace}/");
        debug!("Fetching Docker Hub org info: {url}");

        let resp = self.client.get(&url).send().await?;

        match resp.status() {
            StatusCode::OK => {
                let body: DockerHubOrgResponse = resp.json().await?;
                Ok(body
                    .gravatar_url
                    .filter(|u| !u.is_empty())
                    .map(|url| IconSource::DockerHubOrgGravatar { url }))
            }
            StatusCode::TOO_MANY_REQUESTS => {
                warn!("Rate limited by Docker Hub API");
                Err(Error::RateLimited)
            }
            _ => Ok(None),
        }
    }

    /// Fetch icon from devicons/devicon via jsDelivr CDN
    #[cfg(feature = "devicon")]
    async fn fetch_devicon_logo(&self, image_name: &str) -> Result<Option<IconSource>> {
        let url = format!(
            "https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/{name}/{name}-original.svg",
            name = image_name
        );

        debug!("Checking devicon: {url}");

        let resp = self.client.head(&url).send().await?;
        if resp.status().is_success() {
            Ok(Some(IconSource::Devicon { url }))
        } else {
            Ok(None)
        }
    }

    /// Fetch Docker Official Image logo from docker-library/docs via jsDelivr
    async fn fetch_docker_official_logo(
        &self,
        image_name: &str,
    ) -> Result<Option<IconSource>> {
        let url = format!(
            "https://cdn.jsdelivr.net/gh/docker-library/docs@master/{image_name}/logo.png"
        );

        debug!("Checking Docker Official Image logo: {url}");

        let resp = self.client.head(&url).send().await?;
        if resp.status().is_success() {
            Ok(Some(IconSource::DockerOfficialImage { url }))
        } else {
            Ok(None)
        }
    }

    /// Fetch GitHub avatar for a GHCR image owner
    async fn fetch_ghcr_avatar(&self, namespace: &str) -> Result<Option<IconSource>> {
        let url = format!("https://avatars.githubusercontent.com/{namespace}");
        debug!("Checking GitHub avatar: {url}");

        let resp = self.client.head(&url).send().await?;
        if resp.status().is_success() {
            Ok(Some(IconSource::GhcrAvatar { url }))
        } else {
            Ok(None)
        }
    }
}
