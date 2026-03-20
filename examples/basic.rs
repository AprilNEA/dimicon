use dimicon::{IconService, IconSource, ImageReference};

#[tokio::main]
async fn main() -> Result<(), dimicon::Error> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::DEBUG.into()),
        )
        .init();

    let service = IconService::new();

    // Test different types of images
    let images = [
        "nginx",                         // Docker Official Image
        "redis",                         // Docker Official Image
        "haproxy",                       // Docker Official Image
        "localstack/localstack",         // User image with logo
        "tensorflow/tensorflow",         // Org image
        "ghcr.io/corespeed-io/example",  // GHCR image
    ];

    println!("=== dimicon - Docker Image Icon ===\n");

    for image in &images {
        print!("Fetching icon for: {} ... ", image);

        match service.get_icon(image).await {
            Ok(icon) => {
                match &icon {
                    IconSource::DockerHubLogo { url } => {
                        println!("✓ Docker Hub Logo");
                        println!("  URL: {}", url);
                    }
                    IconSource::DockerHubOrgGravatar { url } => {
                        println!("✓ Docker Hub Org Gravatar");
                        println!("  URL: {}", url);
                    }
                    #[cfg(feature = "devicon")]
                    IconSource::Devicon { url } => {
                        println!("✓ Devicon");
                        println!("  URL: {}", url);
                    }
                    IconSource::DockerOfficialImage { url } => {
                        println!("✓ Docker Official Image");
                        println!("  URL: {}", url);
                    }
                    IconSource::GhcrAvatar { url } => {
                        println!("✓ GHCR Avatar");
                        println!("  URL: {}", url);
                    }
                    IconSource::Custom { url } => {
                        println!("✓ Custom");
                        println!("  URL: {}", url);
                    }
                    IconSource::NotFound => {
                        println!("✗ Not Found");
                    }
                }
            }
            Err(e) => {
                println!("✗ Error: {}", e);
            }
        }
        println!();
    }

    // Demo mirror reference parsing
    println!("=== Image Reference Parsing ===\n");

    let refs = [
        "nginx",
        "nginx:latest",
        "myuser/myimage:v1.0",
        "ghcr.io/owner/app:latest",
        "registry.example.com/ns/image:tag",
        "nginx@sha256:abc123",
    ];

    for r in &refs {
        match ImageReference::parse(r) {
            Ok(parsed) => {
                println!("Input:     {}", r);
                println!("  Registry:  {}", parsed.registry);
                println!("  Namespace: {}", parsed.namespace);
                println!("  Name:      {}", parsed.name);
                println!("  Tag:       {:?}", parsed.tag);
                println!("  Official:  {}", parsed.is_docker_official());
                println!("  GHCR:      {}", parsed.is_ghcr());
                println!();
            }
            Err(e) => {
                println!("Input: {} -> Error: {}", r, e);
            }
        }
    }

    Ok(())
}
