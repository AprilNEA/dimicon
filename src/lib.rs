//! # dimicon — Docker Image Icon
//!
//! A Rust library for fetching Docker image icons from various sources.
//!
//! ## Features
//!
//! - Fetch icons from Docker Hub image logos
//! - Fetch icons from Docker Hub organization Gravatars
//! - Fetch icons from [devicons/devicon](https://github.com/devicons/devicon) (via jsDelivr CDN)
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
//!     if let Some(icon) = service.get_icon("nginx").await? {
//!         println!("nginx icon: {}", icon.url());
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Image Reference Parsing
//!
//! ```
//! use dimicon::ImageReference;
//!
//! let img = ImageReference::parse("nginx").unwrap();
//! assert!(img.is_docker_official());
//!
//! let img = ImageReference::parse("nginx:latest").unwrap();
//! assert_eq!(img.tag(), Some("latest"));
//!
//! let img = ImageReference::parse("myuser/myimage:v1.0").unwrap();
//! assert_eq!(img.namespace(), "myuser");
//!
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
//! | Other registries | Not supported (returns `None`) |

mod error;
mod parser;
mod service;
mod types;

pub use error::{Error, Result};
pub use parser::ImageReference;
pub use service::IconService;
pub use types::IconSource;

/// Convenience function to get an icon for an image
///
/// Creates a new [`IconService`] for each call. For better performance
/// when fetching multiple icons, create a single service and reuse it.
///
/// # Example
///
/// ```no_run
/// #[tokio::main]
/// async fn main() {
///     if let Some(icon) = dimicon::get_icon("nginx").await.unwrap() {
///         println!("Icon URL: {}", icon.url());
///     }
/// }
/// ```
pub async fn get_icon(image: &str) -> Result<Option<IconSource>> {
    IconService::new().get_icon(image).await
}
