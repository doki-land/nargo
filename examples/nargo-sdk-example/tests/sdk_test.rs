use nargo_sdk_example::*;
use tempfile::tempdir;

#[test]
fn test_metadata_extraction() {
    let ir = nargo_ir::IRModule::default();
    let result = ApiMetadataExtractor::extract(EXAMPLE_SOURCE, &ir);

    assert!(result.is_ok());
    let metadata = result.unwrap();

    assert!(metadata.endpoints.len() > 0);
    assert!(metadata.types.len() > 0);
}

#[test]
fn test_sdk_generation() {
    let temp_dir = tempdir().unwrap();
    let output_dir = temp_dir.path();

    let ir = nargo_ir::IRModule::default();
    let metadata = ApiMetadataExtractor::extract(EXAMPLE_SOURCE, &ir).unwrap();

    let result = SdkGenerator::generate(&metadata, output_dir);
    assert!(result.is_ok());

    let types_file = output_dir.join("types.ts");
    let api_file = output_dir.join("api.ts");

    assert!(types_file.exists());
    assert!(api_file.exists());
}

#[test]
fn test_endpoint_parsing() {
    let ir = nargo_ir::IRModule::default();
    let metadata = ApiMetadataExtractor::extract(EXAMPLE_SOURCE, &ir).unwrap();

    let user_endpoints: Vec<_> = metadata.endpoints.iter().filter(|e| e.controller == "UserController").collect();

    assert_eq!(user_endpoints.len(), 5);

    let get_user = user_endpoints.iter().find(|e| e.name == "getUser").unwrap();

    assert_eq!(get_user.method, "GET");
    assert_eq!(get_user.path, "/users/:id");
}
