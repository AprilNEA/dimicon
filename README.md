# dimicon

**D**ocker **Im**age **Icon** - A library for fetching Docker image icons from various sources.

[![Crates.io](https://img.shields.io/crates/v/dimicon.svg)](https://crates.io/crates/dimicon)
[![Documentation](https://docs.rs/dimicon/badge.svg)](https://docs.rs/dimicon)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- Fetch icons from Docker Hub image logos
- Fetch icons from Docker Hub organization Gravatars
- Fetch icons from Docker Official Images (via jsDelivr CDN)
- Fetch icons from GitHub Container Registry (via GitHub Avatar)
- Parse Docker image reference strings in various formats

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
dimicon = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

```rust
use dimicon::IconService;

#[tokio::main]
async fn main() -> Result<(), dimicon::Error> {
    let service = IconService::new();

    // Get icon for an official image
    let icon = service.get_icon("nginx").await?;
    if let Some(url) = icon.url() {
        println!("nginx icon: {}", url);
    }

    // Get icon for a user/org image
    let icon = service.get_icon("localstack/localstack").await?;
    println!("localstack icon: {:?}", icon.url());

    // Get icon for a ghcr.io image
    let icon = service.get_icon("ghcr.io/astral-sh/uv").await?;
    println!("ghcr icon: {:?}", icon.url());

    Ok(())
}
```

## Supported Registries

| Registry | Icon Source |
|----------|-------------|
| Docker Hub (`docker.io`) | Image logo → Org Gravatar → Official Image logo |
| GitHub Container Registry (`ghcr.io`) | GitHub Avatar |
| Other registries | Not supported (returns `NotFound`) |

## Image Reference Parsing

The library can parse various Docker image reference formats:

```rust
use dimicon::ImageReference;

// Simple image name
let img = ImageReference::parse("nginx")?;
assert_eq!(img.registry, "docker.io");
assert_eq!(img.namespace, "library");
assert_eq!(img.name, "nginx");

// Image with tag
let img = ImageReference::parse("nginx:latest")?;
assert_eq!(img.tag, Some("latest".to_string()));

// User/org image
let img = ImageReference::parse("myuser/myimage:v1.0")?;
assert_eq!(img.namespace, "myuser");

// GHCR image
let img = ImageReference::parse("ghcr.io/owner/app:latest")?;
assert!(img.is_ghcr());

// Custom registry
let img = ImageReference::parse("registry.example.com/namespace/image:tag")?;
assert_eq!(img.registry, "registry.example.com");
```

## Icon Sources

The `IconSource` enum represents different icon sources:

```rust
use dimicon::IconSource;

match icon {
    IconSource::DockerHubLogo { url } => println!("Docker Hub logo: {}", url),
    IconSource::DockerHubOrgGravatar { url } => println!("Org gravatar: {}", url),
    IconSource::DockerOfficialImage { url } => println!("Official image: {}", url),
    IconSource::GhcrAvatar { url } => println!("GitHub avatar: {}", url),
    IconSource::Custom { url } => println!("Custom icon: {}", url),
    IconSource::NotFound => println!("No icon found"),
}
```

## API Overview

### `IconService`

The main service for fetching image icons.

```rust
// Create a new service
let service = IconService::new();

// Or with a custom reqwest client
let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(10))
    .build()?;
let service = IconService::with_client(client);

// Fetch icon
let icon = service.get_icon("nginx").await?;
```

### `ImageReference`

Parse and inspect Docker image references.

```rust
let img = ImageReference::parse("ghcr.io/owner/app:v1")?;

img.is_docker_hub()      // false
img.is_ghcr()            // true
img.is_docker_official() // false
img.docker_hub_repo_name() // "owner/app"
img.full_name()          // "ghcr.io/owner/app:v1"
```

### Convenience Function

For one-off lookups:

```rust
let icon = dimicon::get_icon("redis").await?;
```

## How It Works

The library fetches icons using the following priority:

1. **Docker Official Images**: Fetches logos from the [docker-library/docs](https://github.com/docker-library/docs) repository via jsDelivr CDN.

2. **Docker Hub Images**: Queries the Docker Hub media API (`hub.docker.com/api/media/repos_logo/v1/`) for image logos.

3. **Docker Hub Organizations**: Falls back to organization Gravatar via the Docker Hub v2 API.

4. **GitHub Container Registry**: Uses GitHub avatar URLs based on the namespace.

## Rate Limiting

Docker Hub APIs have per-IP rate limits. For production use, consider:

- Caching icon URLs
- Using a proxy service like [go-camo](https://github.com/cactus/go-camo) for image proxying
- Implementing request throttling

## Examples

Run the basic example:

```bash
cargo run --example basic
```

## Credit
[@SukkaW](https://github.com/SukkaW) provided the complete idea for this package.

## License

MIT License - see [LICENSE](LICENSE) for details.
