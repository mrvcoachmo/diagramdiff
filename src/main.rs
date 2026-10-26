mod excalidraw;

use std::{
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
        // The reader stopped early, as `| head` or a closed pager does: nothing went wrong here.
        Err(err) if err.kind() == ErrorKind::BrokenPipe => Ok(()),
        result => result.map_err(|err| err.to_string()),
    }
}

fn read(path: &Path) -> Result<Vec<Node>, String> {
    // Read, not just open: opening a directory succeeds on Unix, reading it fails.
    // Bytes, not a String: a file that is not UTF-8 is still readable.
    let bytes = fs::read(path).map_err(|err| format!("{}: {err}", path.display()))?;
    excalidraw::parse(&bytes)
        .map_err(|err| format!("{}: not an Excalidraw file: {err}", path.display()))
}

/// The Nodes of `a` whose Key is not in `b`, in output order.
fn only_in<'a>(a: &'a [Node], b: &[Node]) -> Vec<&'a Node> {
    let keys: HashSet<&str> = b.iter().map(|node| node.key.as_str()).collect();
    let mut nodes: Vec<&Node> = a
        .iter()
        .filter(|node| !keys.contains(node.key.as_str()))
        .collect();
    // Labelled Nodes by Label as shown, so the lines read in order; then the rest by Key alone.
    // String order is byte order.
    nodes.sort_by_cached_key(|&node| match &node.label {
        Some(label) => (false, shown(label), node.kind.as_str(), node.key.as_str()),
        None => (true, String::new(), "", node.key.as_str()),
    });
    nodes
}

fn line(sign: char, node: &Node) -> String {
    let name = match &node.label {
        Some(label) => format!("\"{}\"", shown(label)),
        None => format!("#{}", plain(&node.key)),
    };
    format!("{sign} {} {name}\n", plain(&node.kind))
}

/// A Label as the output shows it, without its quotes.
fn shown(label: &str) -> String {
    plain(&label.replace("\r\n", "\n").replace('"', "\\\""))
}

/// Control characters, line breaks among them, become spaces: one Change per line,
/// and a diagram cannot send escape codes to the reader's terminal.
fn plain(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}
