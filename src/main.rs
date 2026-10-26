mod excalidraw;

use std::{
    cmp::Ordering,
    collections::HashSet,
    fs,
    io::{self, ErrorKind, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// The old diagram
    old: PathBuf,
    /// The new diagram
    new: PathBuf,
}

struct Node {
    key: String,
    kind: String,
    label: Option<String>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("diagramdiff: {err}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: &Cli) -> Result<(), String> {
    let old = read(&cli.old)?;
    let new = read(&cli.new)?;
    let removed = only_in(&old, &new);
    let added = only_in(&new, &old);
    if removed.is_empty() && added.is_empty() {
        return Ok(());
    }
    let mut out = format!(
        "{} → {}: {} removed, {} added\n",
        cli.old.display(),
        cli.new.display(),
        removed.len(),
        added.len()
    );
    for node in removed {
        out += &line('-', node);
    }
    for node in added {
        out += &line('+', node);
    }
    let mut stdout = io::stdout().lock();
    match stdout
        .write_all(out.as_bytes())
        .and_then(|()| stdout.flush())
    {
        Err(err) if err.kind() == ErrorKind::BrokenPipe => Ok(()),
        result => result.map_err(|err| err.to_string()),
    }
}

fn read(path: &Path) -> Result<Vec<Node>, String> {
    let bytes = fs::read(path).map_err(|err| format!("{}: {err}", path.display()))?;
    excalidraw::parse(&bytes)
        .map_err(|err| format!("{}: not an Excalidraw file: {err}", path.display()))
}

fn only_in<'a>(diagram: &'a [Node], other: &[Node]) -> Vec<&'a Node> {
    let other_keys: HashSet<&str> = other.iter().map(|node| node.key.as_str()).collect();
    let mut nodes: Vec<&Node> = diagram
        .iter()
        .filter(|node| !other_keys.contains(node.key.as_str()))
        .collect();
    nodes.sort_by(|a, b| match (&a.label, &b.label) {
        (Some(a_label), Some(b_label)) => {
            (shown_label(a_label), &a.kind, &a.key).cmp(&(shown_label(b_label), &b.kind, &b.key))
        }
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.key.cmp(&b.key),
    });
    nodes
}

fn line(sign: char, node: &Node) -> String {
    let name = match &node.label {
        Some(label) => format!("\"{}\"", shown_label(label)),
        None => format!("#{}", plain(&node.key)),
    };
    format!("{sign} {} {name}\n", plain(&node.kind))
}

fn shown_label(label: &str) -> String {
    let escaped = label.replace('\\', "\\\\").replace('"', "\\\"");
    plain(&escaped.replace("\r\n", "\n"))
}

// Control characters become spaces: one Change per line, and no escape codes reach the terminal.
fn plain(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}
