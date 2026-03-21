use nargo_formatter::NargoFormatter;

#[tokio::test]
async fn test_format_js() {
    let mut formatter = NargoFormatter::new();

    let source = r#"
const obj = { a: 1, b: 2, c: 3 };
const arr = [1, 2, 3, 4, 5];
const func = (x, y) => x + y;
const cond = true ? 'yes' : 'no';
const template = `Hello ${name}!`;
"#;

    let formatted = formatter.format(source).await.unwrap();
    println!(
        "Formatted JS:
{}",
        formatted
    );

    assert!(formatted.contains("const obj = { a: 1, b: 2, c: 3 }"));
    assert!(formatted.contains("const arr = [1, 2, 3, 4, 5]"));
    assert!(formatted.contains("const func = (x, y) => x + y"));
    assert!(formatted.contains("const cond = true ? 'yes' : 'no'"));
    assert!(formatted.contains("const template = `Hello ${name}!`"));
}

#[tokio::test]
async fn test_format_ts() {
    let mut formatter = NargoFormatter::new();

    let source = r#"
interface User {
    name: string;
    age: number;
}

function greet(user: User): string {
    return `Hello, ${user.name}!`;
}

const user: User = { name: "John", age: 30 };
"#;

    let formatted = formatter.format(source).await.unwrap();
    println!(
        "Formatted TS:
{}",
        formatted
    );

    assert!(formatted.contains("interface User"));
    assert!(formatted.contains("name: string;"));
    assert!(formatted.contains("age: number;"));
    assert!(formatted.contains("function greet(user: User): string"));
    assert!(formatted.contains("const user: User = { name: \"John\", age: 30 }"));
}

#[tokio::test]
async fn test_format_css() {
    let mut formatter = NargoFormatter::new();

    let source = r#"
body {
    margin: 0;
    padding: 0;
    font-family: Arial, sans-serif;
}

.test {
    color: red;
    font-size: 16px;
}
"#;

    let formatted = formatter.format(source).await.unwrap();
    println!(
        "Formatted CSS:
{}",
        formatted
    );

    assert!(formatted.contains("body {"));
    assert!(formatted.contains("margin: 0;"));
    assert!(formatted.contains(".test {"));
    assert!(formatted.contains("color: red;"));
}

#[tokio::test]
async fn test_format_json() {
    let mut formatter = NargoFormatter::new();

    let source = r#"
{
    "name": "John",
    "age": 30,
    "address": {
        "street": "123 Main St",
        "city": "New York"
    }
}
"#;

    let formatted = formatter.format(source).await.unwrap();
    println!(
        "Formatted JSON:
{}",
        formatted
    );

    assert!(formatted.contains("\"name\": \"John\""));
    assert!(formatted.contains("\"age\": 30"));
    assert!(formatted.contains("\"address\":"));
    assert!(formatted.contains("\"street\": \"123 Main St\""));
}
