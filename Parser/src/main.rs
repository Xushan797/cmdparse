use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use tree_sitter::Node;

fn main() -> Result<()> {
    let code = r#"mycmd -f&&yourcmd --bar=foo||bash -c \"ourcmd baz\" --bashflag|&{quux|corge;}"#;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_bash::language()).expect("Error loading Bash grammar");
    let parsed_tree = parser.parse(code, None).unwrap();
    let root_node = parsed_tree.root_node();
    print_node(root_node, code.as_bytes(), 0);
    Ok(())
}

fn print_node(node: tree_sitter::Node, source_code: &[u8], level: usize) {
    let indent = "  ".repeat(level);
    let kind = node.kind();
    let start_byte = node.start_byte();
    let end_byte = node.end_byte();
    let text = &source_code[start_byte..end_byte];
    println!(
        "{}Kind: {}, Text: '{}'",
        indent,
        kind,
        String::from_utf8_lossy(text)
    );
    if node.child_count() > 0 {
        for i in 0..node.child_count() {
            let child = node.child(i).unwrap();
            print_node(child, source_code, level + 1);
        }
    }
}