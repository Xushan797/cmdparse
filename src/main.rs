use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use tree_sitter::Node;

#[derive(clap::Parser)]
struct Cli {
    /// Script file to read, or '-' for stdin
    #[clap(default_value = "-")]
    script: String,

    /// Clean the found commands (coalesce whitespace, normalize quoting)
    #[clap(short = 'c', long)]
    clean: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut code = String::new();
    if cli.script == "-" {
        std::io::stdin().read_to_string(&mut code)?;
    } else {
        let mut file = File::open(&cli.script)?;
        file.read_to_string(&mut code)?;
    }

    let mut commands = parse_and_extract_commands(&code)?;
    let mut final_commands = Vec::new();

    while let Some(command) = commands.pop() {
        match process_embedded_code(&command) {
            Some(inner_commands) => commands.extend(inner_commands),
            None => final_commands.push(command),
        }
    }

    final_commands.reverse();
    for mut command in final_commands {
        if cli.clean {
            command = clean_command(&command);
        }
        println!("{command}");
    }

    Ok(())
}

/// Parse the code string as Bash and extract a list of commands from it
fn parse_and_extract_commands(code: &str) -> Result<Vec<String>> {
    let bash = tree_sitter_bash::language();
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&bash)
        .context("failed to load bash parser")?;
    let tree = parser
        .parse(&code, None)
        .context("failed to parse bash code")?;
    let root_node = tree.root_node();

    let commands = extract_commands(root_node, code);
    Ok(commands)
}

/// Extracts individual commands from Bash code by walking its parse tree.
fn extract_commands(root_node: Node, code: &str) -> Vec<String> {
    let mut commands = Vec::new();

    let mut stack = vec![root_node];
    while let Some(node) = stack.pop() {
        if node.is_named() {
            if node.kind() == "command" {
                let command = &code[node.start_byte()..node.end_byte()];
                commands.push(command.to_string());
            }
        }
        for child in node.named_children(&mut node.walk()) {
            stack.push(child);
        }
    }

    commands.reverse();
    commands
}

/// Attempt to resolve if the command is of the form `sh -c "mycmd ..."`, and if
/// so, unwrap the inner code string
fn process_embedded_code(command: &str) -> Option<Vec<String>> {
    let Some(split) = shlex::split(command) else {
        return None;
    };
    let mut split: VecDeque<String> = split.into();

    let Some(first_arg) = split.pop_front() else {
        return None;
    };
    if &first_arg != "bash" && &first_arg != "sh" {
        return None;
    }

    let Some(second_arg) = split.pop_front() else {
        return None;
    };
    if &second_arg != "-c" {
        return None;
    }

    // At this point the remainder must be the embedded Bash code, and the rest of
    // the args do not matter for our purposes
    let Some(code) = split.pop_front() else {
        return None;
    };

    parse_and_extract_commands(&code).ok()
}

/// Squashes escaped newlines and cleans the command with shlex by splitting and
/// then rejoining. This is opinionated, and might not be needed.
fn clean_command(command: &str) -> String {
    let command = command.replace("\\\n", " ");
    let command = command.replace("\\\r\n", " ");
    let command = shlex::split(&command).unwrap_or_default();
    let command = shlex::try_join(command.iter().map(AsRef::as_ref)).unwrap_or_default();
    command
}
