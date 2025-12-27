use crate::error::{Error, Result};

/// Parsed mirror reference
#[derive(Debug, Clone, PartialEq)]
pub struct ImageReference {
    /// Image registry (e.g., docker.io, ghcr.io)
    pub registry: String,
    /// Namespace/username (e.g., library, nginx, corespeed-io)
    pub namespace: String,
    /// Image name
    pub name: String,
    /// Tag (optional)
    pub tag: Option<String>,
}

impl ImageReference {
    /// Parse image reference string
    ///
    /// Supported formats:
    /// - `nginx` -> docker.io/library/nginx
    /// - `nginx:latest` -> docker.io/library/nginx:latest
    /// - `myuser/myimage` -> docker.io/myuser/myimage
    /// - `ghcr.io/owner/image` -> ghcr.io/owner/image
    /// - `registry.example.com/namespace/image:tag`
    ///
    /// # Examples
    ///
    /// ```
    /// use dimicon::ImageReference;
    ///
    /// let ref1 = ImageReference::parse("nginx").unwrap();
    /// assert_eq!(ref1.registry, "docker.io");
    /// assert_eq!(ref1.namespace, "library");
    /// assert_eq!(ref1.name, "nginx");
    ///
    /// let ref2 = ImageReference::parse("ghcr.io/owner/app:v1").unwrap();
    /// assert!(ref2.is_ghcr());
    /// ```
    pub fn parse(image: &str) -> Result<Self> {
        let image = image.trim();
        if image.is_empty() {
            return Err(Error::InvalidImageReference(
                "Empty image reference".to_string(),
            ));
        }

        // Separate tag
        let (image_without_tag, tag) = if let Some(at_pos) = image.rfind('@') {
            // Handle digest format image@sha256:...
            (image[..at_pos].to_string(), None)
        } else if let Some(colon_pos) = image.rfind(':') {
            // Check if it's a port number (registry:port/image format)
            let before_colon = &image[..colon_pos];
            if before_colon.contains('/') || !image[colon_pos + 1..].contains('/') {
                // It's a tag
                (
                    image[..colon_pos].to_string(),
                    Some(image[colon_pos + 1..].to_string()),
                )
            } else {
                // It's a port number
                (image.to_string(), None)
            }
        } else {
            (image.to_string(), None)
        };

        let parts: Vec<&str> = image_without_tag.split('/').collect();

        let (registry, namespace, name) = match parts.len() {
            1 => {
                // nginx -> docker.io/library/nginx
                (
                    "docker.io".to_string(),
                    "library".to_string(),
                    parts[0].to_string(),
                )
            }
            2 => {
                // Check if the first part is a registry
                if parts[0].contains('.') || parts[0].contains(':') || parts[0] == "localhost" {
                    // registry/image -> registry/library/image
                    (
                        parts[0].to_string(),
                        "library".to_string(),
                        parts[1].to_string(),
                    )
                } else {
                    // user/image -> docker.io/user/image
                    (
                        "docker.io".to_string(),
                        parts[0].to_string(),
                        parts[1].to_string(),
                    )
                }
            }
            _ => {
                // registry/namespace/image or more complex formats
                let registry = parts[0].to_string();
                let name = parts.last().unwrap().to_string();
                let namespace = parts[1..parts.len() - 1].join("/");
                (registry, namespace, name)
            }
        };

        Ok(Self {
            registry,
            namespace,
            name,
            tag,
        })
    }

    /// Get the Docker Hub formatted repository name
    ///
    /// For the library namespace, only the image name is returned; otherwise, namespace/name is returned
    pub fn docker_hub_repo_name(&self) -> String {
        if self.namespace == "library" {
            self.name.clone()
        } else {
            format!("{}/{}", self.namespace, self.name)
        }
    }

    /// Check if it is a Docker Hub image
    pub fn is_docker_hub(&self) -> bool {
        self.registry == "docker.io" || self.registry == "index.docker.io"
    }

    /// Check if it is a GitHub Container Registry image
    pub fn is_ghcr(&self) -> bool {
        self.registry == "ghcr.io"
    }

    /// Check if it is a Docker Official Image
    pub fn is_docker_official(&self) -> bool {
        self.is_docker_hub() && self.namespace == "library"
    }

    /// Get the full image reference string
    pub fn full_name(&self) -> String {
        let base = if self.namespace == "library" && self.is_docker_hub() {
            self.name.clone()
        } else if self.is_docker_hub() {
            format!("{}/{}", self.namespace, self.name)
        } else {
            format!("{}/{}/{}", self.registry, self.namespace, self.name)
        };

        match &self.tag {
            Some(tag) => format!("{}:{}", base, tag),
            None => base,
        }
    }
}

impl std::fmt::Display for ImageReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_name())
    }
}

impl std::str::FromStr for ImageReference {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_image() {
        let parsed = ImageReference::parse("nginx").unwrap();
        assert_eq!(parsed.registry, "docker.io");
        assert_eq!(parsed.namespace, "library");
        assert_eq!(parsed.name, "nginx");
        assert!(parsed.tag.is_none());
    }

    #[test]
    fn test_parse_image_with_tag() {
        let parsed = ImageReference::parse("nginx:latest").unwrap();
        assert_eq!(parsed.registry, "docker.io");
        assert_eq!(parsed.namespace, "library");
        assert_eq!(parsed.name, "nginx");
        assert_eq!(parsed.tag, Some("latest".to_string()));
    }

    #[test]
    fn test_parse_user_image() {
        let parsed = ImageReference::parse("myuser/myimage:v1.0").unwrap();
        assert_eq!(parsed.registry, "docker.io");
        assert_eq!(parsed.namespace, "myuser");
        assert_eq!(parsed.name, "myimage");
        assert_eq!(parsed.tag, Some("v1.0".to_string()));
    }

    #[test]
    fn test_parse_ghcr_image() {
        let parsed = ImageReference::parse("ghcr.io/corespeed-io/myapp:latest").unwrap();
        assert_eq!(parsed.registry, "ghcr.io");
        assert_eq!(parsed.namespace, "corespeed-io");
        assert_eq!(parsed.name, "myapp");
        assert_eq!(parsed.tag, Some("latest".to_string()));
        assert!(parsed.is_ghcr());
    }

    #[test]
    fn test_parse_custom_registry() {
        let parsed =
            ImageReference::parse("registry.example.com/namespace/image:tag").unwrap();
        assert_eq!(parsed.registry, "registry.example.com");
        assert_eq!(parsed.namespace, "namespace");
        assert_eq!(parsed.name, "image");
        assert_eq!(parsed.tag, Some("tag".to_string()));
    }

    #[test]
    fn test_parse_digest() {
        let parsed = ImageReference::parse("nginx@sha256:abc123").unwrap();
        assert_eq!(parsed.name, "nginx");
        assert!(parsed.tag.is_none());
    }

    #[test]
    fn test_parse_registry_with_port() {
        let parsed = ImageReference::parse("localhost:5000/myimage").unwrap();
        assert_eq!(parsed.registry, "localhost:5000");
        assert_eq!(parsed.name, "myimage");
    }

    #[test]
    fn test_is_docker_official() {
        let nginx = ImageReference::parse("nginx").unwrap();
        assert!(nginx.is_docker_official());

        let user_image = ImageReference::parse("myuser/myimage").unwrap();
        assert!(!user_image.is_docker_official());
    }

    #[test]
    fn test_docker_hub_repo_name() {
        let nginx = ImageReference::parse("nginx").unwrap();
        assert_eq!(nginx.docker_hub_repo_name(), "nginx");

        let user_image = ImageReference::parse("myuser/myimage").unwrap();
        assert_eq!(user_image.docker_hub_repo_name(), "myuser/myimage");
    }

    #[test]
    fn test_full_name() {
        assert_eq!(ImageReference::parse("nginx").unwrap().full_name(), "nginx");
        assert_eq!(
            ImageReference::parse("nginx:latest").unwrap().full_name(),
            "nginx:latest"
        );
        assert_eq!(
            ImageReference::parse("user/app").unwrap().full_name(),
            "user/app"
        );
        assert_eq!(
            ImageReference::parse("ghcr.io/owner/app:v1").unwrap().full_name(),
            "ghcr.io/owner/app:v1"
        );
    }

    #[test]
    fn test_parse_empty_string() {
        assert!(ImageReference::parse("").is_err());
        assert!(ImageReference::parse("   ").is_err());
    }

    #[test]
    fn test_from_str() {
        let parsed: ImageReference = "nginx:latest".parse().unwrap();
        assert_eq!(parsed.name, "nginx");
        assert_eq!(parsed.tag, Some("latest".to_string()));
    }
}
