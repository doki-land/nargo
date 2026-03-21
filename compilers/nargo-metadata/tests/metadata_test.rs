use nargo_ir::IRModule;
use nargo_metadata::{ApiMetadataExtractor, ApiParamType};

#[test]
fn test_extract_simple_endpoint() {
    let source = r#"
class UserController {
    @http("GET", "/users/:id")
    async getUser(@Path id: number): Promise<User> {
    }
}
"#;

    let ir = IRModule::default();
    let metadata = ApiMetadataExtractor::extract(source, &ir).unwrap();

    assert_eq!(metadata.endpoints.len(), 1);

    let endpoint = &metadata.endpoints[0];
    assert_eq!(endpoint.controller, "UserController");
    assert_eq!(endpoint.name, "getUser");
    assert_eq!(endpoint.method, "GET");
    assert_eq!(endpoint.path, "/users/:id");
    assert_eq!(endpoint.is_async, true);
    assert_eq!(endpoint.params.len(), 1);

    let param = &endpoint.params[0];
    assert_eq!(param.name, "id");
    assert_eq!(param.param_type, ApiParamType::Path);
    assert_eq!(param.type_name, "number");
}

#[test]
fn test_extract_multiple_endpoints() {
    let source = r#"
class UserController {
    @http("GET", "/users")
    async listUsers(): Promise<User[]> {
    }

    @http("POST", "/users")
    async createUser(@Body user: CreateUserDto): Promise<User> {
    }
}
"#;

    let ir = IRModule::default();
    let metadata = ApiMetadataExtractor::extract(source, &ir).unwrap();

    assert_eq!(metadata.endpoints.len(), 2);
}

#[test]
fn test_extract_interface() {
    let source = r#"
interface User {
    id: number;
    name: string;
    email: string;
    optional?: string;
}
"#;

    let ir = IRModule::default();
    let metadata = ApiMetadataExtractor::extract(source, &ir).unwrap();

    assert!(metadata.types.contains_key("User"));

    let user_type = metadata.types.get("User").unwrap();
    assert_eq!(user_type.name, "User");
    assert_eq!(user_type.fields.len(), 4);

    assert_eq!(user_type.fields[0].name, "id");
    assert_eq!(user_type.fields[0].type_name, "number");
    assert_eq!(user_type.fields[0].optional, false);

    assert_eq!(user_type.fields[3].name, "optional");
    assert_eq!(user_type.fields[3].optional, true);
}

#[test]
fn test_extract_class() {
    let source = r#"
class User {
    id: number;
    name: string;
}

class UserController {
    @http("GET", "/users")
    async getUsers(): Promise<User[]> {
    }
}
"#;

    let ir = IRModule::default();
    let metadata = ApiMetadataExtractor::extract(source, &ir).unwrap();

    assert!(!metadata.types.contains_key("UserController"));
    assert!(metadata.types.contains_key("User"));
}

#[test]
fn test_return_type() {
    let source = r#"
class UserController {
    @http("GET", "/users/:id")
    async getUser(@Path id: number): Promise<User> {
    }

    @http("DELETE", "/users/:id")
    async deleteUser(@Path id: number): Promise<void> {
    }
}
"#;

    let ir = IRModule::default();
    let metadata = ApiMetadataExtractor::extract(source, &ir).unwrap();

    assert_eq!(metadata.endpoints[0].return_type, Some("Promise<User>".to_string()));
    assert_eq!(metadata.endpoints[1].return_type, Some("Promise<void>".to_string()));
}

#[test]
fn test_extractor_param() {
    let source = r#"
class UserController {
    @http("GET", "/users/:userId")
    async getUser(@Path("userId") id: number): Promise<User> {
    }
}
"#;

    let ir = IRModule::default();
    let metadata = ApiMetadataExtractor::extract(source, &ir).unwrap();

    assert_eq!(metadata.endpoints.len(), 1);
    assert_eq!(metadata.endpoints[0].params[0].extractor_param, Some("userId".to_string()));
}

#[test]
fn test_empty_source() {
    let source = "";

    let ir = IRModule::default();
    let metadata = ApiMetadataExtractor::extract(source, &ir).unwrap();

    assert_eq!(metadata.endpoints.len(), 0);
    assert_eq!(metadata.types.len(), 0);
}
