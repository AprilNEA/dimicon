//! # dimicon - Docker Image Icon
//!
//! A Rust library for fetching Docker image icons from various sources.
//!
//! ## Features
//!
//! - Fetch icons from Docker Hub image logos
//! - Fetch icons from Docker Hub organization Gravatars
//! - Fetch icons from Docker Official Images (via jsDelivr CDN)
//! - Fetch icons from GitHub Container Registry (via GitHub Avatar)
//! - Parse Docker image reference strings
//!
//! ## Quick Start
//!
//! ```no_run
//! use dimicon::IconService;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), dimicon::Error> {
//!     let service = IconService::new();
//!
//!     // Get icon for an official image
//!     let icon = service.get_icon("nginx").await?;
//!     println!("nginx icon: {:?}", icon.url());
//!
//!     // Get icon for a user image
//!     let icon = service.get_icon("localstack/localstack").await?;
//!     println!("localstack icon: {:?}", icon.url());
//!
//!     // Get icon for a ghcr.io image
//!     let icon = service.get_icon("ghcr.io/corespeed-io/myapp").await?;
//!     println!("ghcr icon: {:?}", icon.url());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Image Reference Parsing
//!
//! The library can parse various Docker image reference formats:
//!
//! ```
//! use dimicon::ImageReference;
//!
//! // Simple image name
//! let img = ImageReference::parse("nginx").unwrap();
//! assert!(img.is_docker_official());
//!
//! // Image with tag
//! let img = ImageReference::parse("nginx:latest").unwrap();
//! assert_eq!(img.tag, Some("latest".to_string()));
//!
//! // User/org image
//! let img = ImageReference::parse("myuser/myimage:v1.0").unwrap();
//! assert_eq!(img.namespace, "myuser");
//!
//! // GHCR image
//! let img = ImageReference::parse("ghcr.io/owner/app:latest").unwrap();
//! assert!(img.is_ghcr());
//! ```
//!
//! ## Supported Registries
//!
//! | Registry | Icon Source |
//! |----------|-------------|
//! | Docker Hub (docker.io) | Image logo, Org Gravatar, Official Image logo |
//! | GitHub Container Registry (ghcr.io) | GitHub Avatar |
//! | Other registries | Not supported (returns `NotFound`) |

mod error;
mod parser;
mod service;
mod types;

pub use error::{Error, Result};
pub use parser::ImageReference;
pub use service::IconService;
pub use types::{DockerHubLogoResponse, DockerHubOrgResponse, DockerHubUserResponse, IconSource};

/// Convenience function to get an icon for an image
///
/// This creates a new `IconService` instance for each call.
/// For better performance when fetching multiple icons, create
/// a single `IconService` and reuse it.
///
/// # Example
///
/// ```no_run
/// #[tokio::main]
/// async fn main() {
///     let icon = dimicon::get_icon("nginx").await.unwrap();
///     println!("Icon URL: {:?}", icon.url());
/// }
/// ```
pub async fn get_icon(image: &str) -> Result<IconSource> {
    IconService::new().get_icon(image).await
}
