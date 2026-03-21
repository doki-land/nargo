# nargo-registry

NPM Registry client and dependency resolver for Nargo.

This crate provides functionality for:
- NPM registry API interaction
- Semantic version parsing and constraint matching
- Git dependency cloning and checkout
- Local path dependency resolution

## Example

```rust,no_run
use nargo_registry::{RegistryClient, RegistryConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RegistryClient::new()?;
    
    let metadata = client.get_package_metadata("lodash").await?;
    println!("Package: {}", metadata.name);
    println!("Versions: {:?}", metadata.versions.keys().collect::<Vec<_>>());
    
    Ok(())
}
```
