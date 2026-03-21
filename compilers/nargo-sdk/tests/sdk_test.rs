use nargo_ir::IRModule;
use nargo_metadata::{ApiEndpoint, ApiMetadata, ApiMetadataExtractor, ApiParam, ApiParamType, TypeDefinition, TypeField};
use nargo_sdk::SdkGenerator;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_generate_types() {
    let mut metadata = ApiMetadata::default();

    let mut user_type = TypeDefinition::default();
    user_type.name = "User".to_string();
    user_type.fields.push(TypeField { name: "id".to_string(), type_name: "number".to_string(), optional: false });
    user_type.fields.push(TypeField { name: "name".to_string(), type_name: "string".to_string(), optional: false });
    user_type.fields.push(TypeField { name: "email".to_string(), type_name: "string".to_string(), optional: true });

    metadata.types.insert("User".to_string(), user_type);

    let temp_dir = tempdir().unwrap();
    let output_dir = temp_dir.path();

    SdkGenerator::generate(&metadata, output_dir).unwrap();

    let types_path = output_dir.join("types.ts");
    assert!(types_path.exists());

    let types_content = fs::read_to_string(types_path).unwrap();
    assert!(types_content.contains("export interface User"));
    assert!(types_content.contains("id: number"));
    assert!(types_content.contains("name: string"));
    assert!(types_content.contains("email?: string"));
}

#[test]
fn test_generate_api_client() {
    let mut metadata = ApiMetadata::default();

    let mut endpoint = ApiEndpoint::default();
    endpoint.controller = "UserController".to_string();
    endpoint.name = "getUser".to_string();
    endpoint.method = "GET".to_string();
    endpoint.path = "/users/:id".to_string();
    endpoint.return_type = Some("Promise<User>".to_string());
    endpoint.is_async = true;

    let mut param = ApiParam::default();
    param.name = "id".to_string();
    param.param_type = ApiParamType::Path;
    param.type_name = "number".to_string();
    endpoint.params.push(param);

    metadata.endpoints.push(endpoint);

    let temp_dir = tempdir().unwrap();
    let output_dir = temp_dir.path();

    SdkGenerator::generate(&metadata, output_dir).unwrap();

    let api_path = output_dir.join("api.ts");
    assert!(api_path.exists());

    let api_content = fs::read_to_string(api_path).unwrap();
    assert!(api_content.contains("export const UserController"));
    assert!(api_content.contains("async getUser(id: number)"));
    assert!(api_content.contains("getUser as any)._nargo_id"));
    assert!(api_content.contains("getUser as any).url"));
    assert!(api_content.contains("getUser as any).method"));
}

#[test]
fn test_generate_multiple_controllers() {
    let mut metadata = ApiMetadata::default();

    let user_endpoint = ApiEndpoint { controller: "UserController".to_string(), name: "getUser".to_string(), method: "GET".to_string(), path: "/users/:id".to_string(), params: vec![], return_type: None, is_async: true };

    let post_endpoint = ApiEndpoint { controller: "PostController".to_string(), name: "getPost".to_string(), method: "GET".to_string(), path: "/posts/:id".to_string(), params: vec![], return_type: None, is_async: true };

    metadata.endpoints.push(user_endpoint);
    metadata.endpoints.push(post_endpoint);

    let temp_dir = tempdir().unwrap();
    let output_dir = temp_dir.path();

    SdkGenerator::generate(&metadata, output_dir).unwrap();

    let api_content = fs::read_to_string(output_dir.join("api.ts")).unwrap();
    assert!(api_content.contains("export const UserController"));
    assert!(api_content.contains("export const PostController"));
}

#[test]
fn test_generate_with_query_params() {
    let mut metadata = ApiMetadata::default();

    let mut endpoint = ApiEndpoint::default();
    endpoint.controller = "UserController".to_string();
    endpoint.name = "listUsers".to_string();
    endpoint.method = "GET".to_string();
    endpoint.path = "/users".to_string();
    endpoint.is_async = true;

    let page_param = ApiParam { name: "page".to_string(), param_type: ApiParamType::Query, type_name: "number".to_string(), extractor_param: None };

    let limit_param = ApiParam { name: "limit".to_string(), param_type: ApiParamType::Query, type_name: "number".to_string(), extractor_param: None };

    endpoint.params.push(page_param);
    endpoint.params.push(limit_param);
    metadata.endpoints.push(endpoint);

    let temp_dir = tempdir().unwrap();
    let output_dir = temp_dir.path();

    SdkGenerator::generate(&metadata, output_dir).unwrap();

    let api_content = fs::read_to_string(output_dir.join("api.ts")).unwrap();
    assert!(api_content.contains("page: number"));
    assert!(api_content.contains("limit: number"));
    assert!(api_content.contains("params: { page, limit }"));
}

#[test]
fn test_generate_with_body_param() {
    let mut metadata = ApiMetadata::default();

    let mut endpoint = ApiEndpoint::default();
    endpoint.controller = "UserController".to_string();
    endpoint.name = "createUser".to_string();
    endpoint.method = "POST".to_string();
    endpoint.path = "/users".to_string();
    endpoint.is_async = true;

    let body_param = ApiParam { name: "user".to_string(), param_type: ApiParamType::Body, type_name: "CreateUserDto".to_string(), extractor_param: None };

    endpoint.params.push(body_param);
    metadata.endpoints.push(endpoint);

    let temp_dir = tempdir().unwrap();
    let output_dir = temp_dir.path();

    SdkGenerator::generate(&metadata, output_dir).unwrap();

    let api_content = fs::read_to_string(output_dir.join("api.ts")).unwrap();
    assert!(api_content.contains("user: CreateUserDto"));
    assert!(api_content.contains("client.post(`/users`, user"));
}

#[test]
fn test_integration_with_metadata_extractor() {
    let source = r#"
class UserController {
    @http("GET", "/users/:id")
    async getUser(@Path id: number): Promise<User> {
    }
}

interface User {
    id: number;
    name: string;
    email: string;
}
"#;

    let ir = IRModule::default();
    let metadata = ApiMetadataExtractor::extract(source, &ir).unwrap();

    let temp_dir = tempdir().unwrap();
    let output_dir = temp_dir.path();

    SdkGenerator::generate(&metadata, output_dir).unwrap();

    assert!(output_dir.join("types.ts").exists());
    assert!(output_dir.join("api.ts").exists());

    let types_content = fs::read_to_string(output_dir.join("types.ts")).unwrap();
    assert!(types_content.contains("export interface User"));

    let api_content = fs::read_to_string(output_dir.join("api.ts")).unwrap();
    assert!(api_content.contains("export const UserController"));
}

#[test]
fn test_creates_output_directory() {
    let metadata = ApiMetadata::default();
    let temp_dir = tempdir().unwrap();
    let nested_dir = temp_dir.path().join("deep").join("nested").join("dir");

    assert!(!nested_dir.exists());

    SdkGenerator::generate(&metadata, &nested_dir).unwrap();

    assert!(nested_dir.exists());
}
