use nargo_ir::TemplateNodeIR;
use nargo_parser::{ParseState, TemplateParser, template::VueTemplateParser};

fn parse(source: &str) -> Vec<TemplateNodeIR> {
    let mut state = ParseState::new(source);
    let parser = VueTemplateParser;
    parser.parse(&mut state, "html").unwrap()
}

#[test]
fn test_parse_basic() {
    let source = r#"<div class="container">Hello {{ name }}!</div>"#;
    let nodes = parse(source);
    assert_eq!(nodes.len(), 1);
    if let TemplateNodeIR::Element(el) = &nodes[0] {
        assert_eq!(el.tag, "div");
        assert_eq!(el.attributes.iter().find(|a| a.name == "class").unwrap().value.as_ref().unwrap(), "container");
        assert_eq!(el.children.len(), 3);
        if let TemplateNodeIR::Interpolation(expr) = &el.children[1] {
            assert_eq!(expr.code, "name");
        }
        else {
            panic!("Expected interpolation");
        }
    }
}

#[test]
fn test_parse_void_elements() {
    let source = r#"<div><br><img src="test.png"></div>"#;
    let nodes = parse(source);
    assert_eq!(nodes.len(), 1);
    if let TemplateNodeIR::Element(el) = &nodes[0] {
        // VueOaksParser handles void elements based on the grammar
        assert_eq!(el.children.len(), 2);
        if let TemplateNodeIR::Element(br) = &el.children[0] {
            assert_eq!(br.tag, "br");
            assert!(br.children.is_empty());
        }
    }
}

#[test]
fn test_parse_comment() {
    let source = r#"<div><!-- this is a comment --></div>"#;
    let nodes = parse(source);
    if let TemplateNodeIR::Element(el) = &nodes[0] {
        if let TemplateNodeIR::Comment(c, _, _) = &el.children[0] {
            assert_eq!(c, " this is a comment ");
        }
    }
}

#[test]
fn test_parse_unquoted_attr() {
    let source = r#"<div class=container id=main></div>"#;
    let nodes = parse(source);
    assert_eq!(nodes.len(), 1);
    if let TemplateNodeIR::Element(el) = &nodes[0] {
        assert_eq!(el.attributes.iter().find(|a| a.name == "class").unwrap().value.as_ref().unwrap(), "container");
        assert_eq!(el.attributes.iter().find(|a| a.name == "id").unwrap().value.as_ref().unwrap(), "main");
    }
}
