use dimicon::{IconService, ImageReference};

#[tokio::main]
async fn main() -> Result<(), dimicon::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::DEBUG.into()),
        )
        .init();

    let service = IconService::new();

    let images = [
        "nginx",
        "redis",
        "haproxy",
        "localstack/localstack",
        "tensorflow/tensorflow",
        "ghcr.io/corespeed-io/example",
    ];

    println!("=== dimicon — Docker Image Icon ===\n");

    for image in &images {
        print!("Fetching icon for: {image} ... ");

        match service.get_icon(image).await {
            Ok(Some(icon)) => {
                println!("✓ Found");
                println!("  URL: {}", icon.url());
            }
            Ok(None) => println!("✗ Not Found"),
            Err(e) => println!("✗ Error: {e}"),
        }
        println!();
    }

    println!("=== Image Reference Parsing ===\n");

    let refs = [
        "nginx",
        "nginx:latest",
        "myuser/myimage:v1.0",
        "myorg/team/app",
        "ghcr.io/owner/app:latest",
        "registry.example.com/ns/image:tag",
        "nginx@sha256:abc123",
    ];

    for input in &refs {
        match ImageReference::parse(input) {
            Ok(r) => {
                println!("Input:     {input}");
                println!("  Registry:  {}", r.registry());
                println!("  Namespace: {}", r.namespace());
                println!("  Name:      {}", r.name());
                println!("  Tag:       {:?}", r.tag());
                println!("  Digest:    {:?}", r.digest());
                println!("  Official:  {}", r.is_docker_official());
                println!("  GHCR:      {}", r.is_ghcr());
                println!();
            }
            Err(e) => println!("Input: {input} → Error: {e}\n"),
        }
    }

    Ok(())
}
