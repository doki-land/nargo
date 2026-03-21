use oak_dejavu::ast::ItemNode;

#[test]
fn test_dejavu_children_detailed() {
    let source = "<% if show %>yes<% else %>no<% end if %>";
    let parse_output = oak_dejavu::parse(source).unwrap();
    
    println!("Source: {}", source);
    println!("\nAST items: {} items", parse_output.items.len());
    
    for (i, item) in parse_output.items.iter().enumerate() {
        println!("Item {}: {:?}", i, item);
    }
    
    if let ItemNode::IfControl(if_ctrl) = &parse_output.items[0] {
        println!("\n=== IfControl Analysis ===");
        println!("condition: {:?}", if_ctrl.condition);
        println!("\nthen_body ({} items):", if_ctrl.then_body.len());
        for (i, item) in if_ctrl.then_body.iter().enumerate() {
            println!("  [{}] {:?}", i, item);
        }
        println!("\nelse_branch: {:?}", if_ctrl.else_branch);
    }
}
