use std::borrow::Cow;

use crate::error::{Error, Result};

/// A parsed Docker image reference
///
/// Supports formats like:
/// - `nginx` → `docker.io/library/nginx`
/// - `nginx:latest` → `docker.io/library/nginx:latest`
/// - `myuser/myimage` → `docker.io/myuser/myimage`
/// - `myorg/team/app` → `docker.io/myorg/team/app`
/// - `ghcr.io/owner/image` → `ghcr.io/owner/image`
/// - `registry.example.com/namespace/image:tag`
/// - `nginx@sha256:abc123` → digest reference
///
/// # Examples
///
/// ```
/// use dimicon::ImageReference;
///
/// let img = ImageReference::parse("nginx").unwrap();
/// assert_eq!(img.registry(), "docker.io");
/// assert_eq!(img.namespace(), "library");
/// assert_eq!(img.name(), "nginx");
///
/// let img = ImageReference::parse("ghcr.io/owner/app:v1").unwrap();
/// assert!(img.is_ghcr());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageReference {
    registry: String,
    namespace: String,
    name: String,
    tag: Option<String>,
    digest: Option<String>,
}

impl ImageReference {
    /// Parse an image reference string
    pub fn parse(image: &str) -> Result<Self> {
        let input = image.trim();
        if input.is_empty() {
            return Err(Error::InvalidImageReference("empty image reference"));
        }

        // 1. Separate digest (@sha256:...)
        let (without_digest, digest) = match input.rsplit_once('@') {
            Some((before, after)) if !after.is_empty() => {
                (before, Some(after.to_owned()))
            }
            Some(_) => return Err(Error::InvalidImageReference("empty digest")),
            None => (input, None),
        };

        // 2. Separate tag
        let (without_tag, tag) = match without_digest.rsplit_once(':') {
            Some((before, after)) => {
                // Disambiguate tag vs port:
                //   `user/image:tag`        → before contains '/', it's a tag
                //   `image:tag`             → after has no '/', it's a tag
                //   `localhost:5000/image`  → after contains '/', it's a port
                if before.contains('/') || !after.contains('/') {
                    if after.is_empty() {
                        return Err(Error::InvalidImageReference("empty tag"));
                    }
                    (before, Some(after.to_owned()))
                } else {
                    (without_digest, None)
                }
            }
            None => (without_digest, None),
        };

        // 3. Split path segments and validate
        let parts: Vec<&str> = without_tag.split('/').collect();
        if parts.iter().any(|p| p.is_empty()) {
            return Err(Error::InvalidImageReference("empty path segment"));
        }

        let is_registry =
            |s: &str| s.contains('.') || s.contains(':') || s == "localhost";

        let (registry, namespace, name) = match parts.len() {
            1 => (
                "docker.io".to_owned(),
                "library".to_owned(),
                parts[0].to_owned(),
            ),
            2 => {
                if is_registry(parts[0]) {
                    (parts[0].to_owned(), "library".to_owned(), parts[1].to_owned())
                } else {
                    ("docker.io".to_owned(), parts[0].to_owned(), parts[1].to_owned())
                }
            }
            _ => {
                // 3+ segments: first is registry only if it looks like one
                let name = (*parts.last().unwrap()).to_owned();
                if is_registry(parts[0]) {
                    let namespace = parts[1..parts.len() - 1].join("/");
                    (parts[0].to_owned(), namespace, name)
                } else {
                    let namespace = parts[..parts.len() - 1].join("/");
                    ("docker.io".to_owned(), namespace, name)
                }
            }
        };

        Ok(Self { registry, namespace, name, tag, digest })
    }

    /// Image registry (e.g., `docker.io`, `ghcr.io`)
    pub fn registry(&self) -> &str {
        &self.registry
    }

    /// Namespace or username (e.g., `library`, `myuser`, `myorg/team`)
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Image name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Tag, if present
    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }

    /// Digest, if present (e.g., `sha256:abc123`)
    pub fn digest(&self) -> Option<&str> {
        self.digest.as_deref()
    }

    /// Repository name formatted for the Docker Hub API
    ///
    /// Returns just the image name for `library/*`, otherwise `namespace/name`.
    pub fn docker_hub_repo_name(&self) -> Cow<'_, str> {
        if self.namespace == "library" {
            Cow::Borrowed(&self.name)
        } else {
            Cow::Owned(format!("{}/{}", self.namespace, self.name))
        }
    }

    /// Check if this is a Docker Hub image
    pub fn is_docker_hub(&self) -> bool {
        self.registry == "docker.io" || self.registry == "index.docker.io"
    }

    /// Check if this is a GitHub Container Registry image
    pub fn is_ghcr(&self) -> bool {
        self.registry == "ghcr.io"
    }

    /// Check if this is a Docker Official Image (`docker.io/library/*`)
    pub fn is_docker_official(&self) -> bool {
        self.is_docker_hub() && self.namespace == "library"
    }
}

impl std::fmt::Display for ImageReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_docker_hub() {
            if self.namespace == "library" {
                write!(f, "{}", self.name)?;
            } else {
                write!(f, "{}/{}", self.namespace, self.name)?;
            }
        } else {
            write!(f, "{}/{}/{}", self.registry, self.namespace, self.name)?;
        }
        if let Some(tag) = &self.tag {
            write!(f, ":{tag}")?;
        }
        if let Some(digest) = &self.digest {
            write!(f, "@{digest}")?;
        }
        Ok(())
    }
}

impl std::str::FromStr for ImageReference {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl TryFrom<&str> for ImageReference {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Self::parse(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_image() {
        let r = ImageReference::parse("nginx").unwrap();
        assert_eq!(r.registry(), "docker.io");
        assert_eq!(r.namespace(), "library");
        assert_eq!(r.name(), "nginx");
        assert!(r.tag().is_none());
        assert!(r.digest().is_none());
        assert!(r.is_docker_official());
    }

    #[test]
    fn test_image_with_tag() {
        let r = ImageReference::parse("nginx:latest").unwrap();
        assert_eq!(r.name(), "nginx");
        assert_eq!(r.tag(), Some("latest"));
    }

    #[test]
    fn test_user_image() {
        let r = ImageReference::parse("myuser/myimage:v1.0").unwrap();
        assert_eq!(r.registry(), "docker.io");
        assert_eq!(r.namespace(), "myuser");
        assert_eq!(r.name(), "myimage");
        assert_eq!(r.tag(), Some("v1.0"));
        assert!(!r.is_docker_official());
    }

    #[test]
    fn test_ghcr_image() {
        let r = ImageReference::parse("ghcr.io/corespeed-io/myapp:latest").unwrap();
        assert_eq!(r.registry(), "ghcr.io");
        assert_eq!(r.namespace(), "corespeed-io");
        assert_eq!(r.name(), "myapp");
        assert_eq!(r.tag(), Some("latest"));
        assert!(r.is_ghcr());
    }

    #[test]
    fn test_custom_registry() {
        let r = ImageReference::parse("registry.example.com/namespace/image:tag").unwrap();
        assert_eq!(r.registry(), "registry.example.com");
        assert_eq!(r.namespace(), "namespace");
        assert_eq!(r.name(), "image");
        assert_eq!(r.tag(), Some("tag"));
    }

    #[test]
    fn test_multi_segment_without_registry() {
        let r = ImageReference::parse("myorg/team/app").unwrap();
        assert_eq!(r.registry(), "docker.io");
        assert_eq!(r.namespace(), "myorg/team");
        assert_eq!(r.name(), "app");
    }

    #[test]
    fn test_digest() {
        let r = ImageReference::parse("nginx@sha256:abc123").unwrap();
        assert_eq!(r.name(), "nginx");
        assert!(r.tag().is_none());
        assert_eq!(r.digest(), Some("sha256:abc123"));
    }

    #[test]
    fn test_tag_and_digest() {
        let r = ImageReference::parse("nginx:latest@sha256:abc123").unwrap();
        assert_eq!(r.tag(), Some("latest"));
        assert_eq!(r.digest(), Some("sha256:abc123"));
    }

    #[test]
    fn test_registry_with_port() {
        let r = ImageReference::parse("localhost:5000/myimage").unwrap();
        assert_eq!(r.registry(), "localhost:5000");
        assert_eq!(r.name(), "myimage");
    }

    #[test]
    fn test_docker_hub_repo_name() {
        let nginx = ImageReference::parse("nginx").unwrap();
        assert_eq!(nginx.docker_hub_repo_name().as_ref(), "nginx");

        let user = ImageReference::parse("myuser/myimage").unwrap();
        assert_eq!(user.docker_hub_repo_name().as_ref(), "myuser/myimage");
    }

    #[test]
    fn test_display() {
        assert_eq!(ImageReference::parse("nginx").unwrap().to_string(), "nginx");
        assert_eq!(
            ImageReference::parse("nginx:latest").unwrap().to_string(),
            "nginx:latest"
        );
        assert_eq!(
            ImageReference::parse("user/app").unwrap().to_string(),
            "user/app"
        );
        assert_eq!(
            ImageReference::parse("ghcr.io/owner/app:v1").unwrap().to_string(),
            "ghcr.io/owner/app:v1"
        );
        assert_eq!(
            ImageReference::parse("nginx@sha256:abc").unwrap().to_string(),
            "nginx@sha256:abc"
        );
    }

    #[test]
    fn test_invalid_inputs() {
        assert!(ImageReference::parse("").is_err());
        assert!(ImageReference::parse("   ").is_err());
        assert!(ImageReference::parse("foo/").is_err());
        assert!(ImageReference::parse("/bar").is_err());
        assert!(ImageReference::parse("foo//bar").is_err());
        assert!(ImageReference::parse("nginx:").is_err());
        assert!(ImageReference::parse("nginx@").is_err());
    }

    #[test]
    fn test_from_str() {
        let r: ImageReference = "nginx:latest".parse().unwrap();
        assert_eq!(r.name(), "nginx");
        assert_eq!(r.tag(), Some("latest"));
    }

    #[test]
    fn test_try_from() {
        let r = ImageReference::try_from("nginx").unwrap();
        assert!(r.is_docker_official());
    }
}
