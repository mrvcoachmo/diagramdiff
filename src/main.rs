mod excalidraw;

use std::{
    cmp::Ordering,
    collections::HashSet,
    fs,
    io::{self, ErrorKind, Write},
    iter,
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

struct Diagram {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

#[derive(Clone)]
struct Node {
    key: String,
    kind: String,
    label: Option<String>,
}

struct Edge {
    key: String,
    kind: String,
    label: Option<String>,
    from: Option<Node>,
    to: Option<Node>,
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) if !err.use_stderr() => err.exit(),
        Err(err) => return fail(&plain_lines(err.to_string().lines())),
    };
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => fail(&plain_lines([format!("diagramdiff: {err}")])),
    }
}

// With stderr closed there is nowhere left to report to.
fn fail(text: &str) -> ExitCode {
    let _ = io::stderr().write_all(text.as_bytes());
    ExitCode::from(2)
}

fn run(cli: &Cli) -> Result<(), String> {
    let old = read(&cli.old)?;
    let new = read(&cli.new)?;
    let removed = lines_only_in(&old, &new);
    let added = lines_only_in(&new, &old);
    if removed.is_empty() && added.is_empty() {
        return Ok(());
    }
    let header = format!(
        "{} → {}: {} removed, {} added",
        cli.old.display(),
        cli.new.display(),
        removed.len(),
        added.len()
    );
    let out = plain_lines(
        iter::once(header)
            .chain(removed.iter().map(|change| format!("- {change}")))
            .chain(added.iter().map(|change| format!("+ {change}"))),
    );
    let mut stdout = io::stdout().lock();
    match stdout
        .write_all(out.as_bytes())
        .and_then(|()| stdout.flush())
    {
        Err(err) if err.kind() == ErrorKind::BrokenPipe => Ok(()),
        result => result.map_err(|err| format!("stdout: {err}")),
    }
}

fn read(path: &Path) -> Result<Diagram, String> {
    let bytes = fs::read(path).map_err(|err| format!("{}: {err}", path.display()))?;
    excalidraw::parse(&bytes)
        .map_err(|err| format!("{}: not an Excalidraw file: {err}", path.display()))
}

fn lines_only_in(diagram: &Diagram, other: &Diagram) -> Vec<String> {
    let other_keys: HashSet<&str> = other
        .nodes
        .iter()
        .map(|node| node.key.as_str())
        .chain(other.edges.iter().map(|edge| edge.key.as_str()))
        .collect();
    let mut nodes: Vec<&Node> = diagram
        .nodes
        .iter()
        .filter(|node| !other_keys.contains(node.key.as_str()))
        .collect();
    nodes.sort_by(|a, b| by_name(a, b).then_with(|| (&a.kind, &a.key).cmp(&(&b.kind, &b.key))));
    let mut edges: Vec<&Edge> = diagram
        .edges
        .iter()
        .filter(|edge| !other_keys.contains(edge.key.as_str()))
        .collect();
    edges.sort_by(|a, b| {
        by_end(a.from.as_ref(), b.from.as_ref())
            .then_with(|| by_end(a.to.as_ref(), b.to.as_ref()))
            .then_with(|| {
                a.label
                    .as_deref()
                    .map(shown_label)
                    .cmp(&b.label.as_deref().map(shown_label))
            })
            .then_with(|| a.key.cmp(&b.key))
    });
    nodes
        .into_iter()
        .map(node_line)
        .chain(edges.into_iter().map(edge_line))
        .collect()
}

fn by_name(a: &Node, b: &Node) -> Ordering {
    match (&a.label, &b.label) {
        (Some(a_label), Some(b_label)) => shown_label(a_label).cmp(&shown_label(b_label)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.key.cmp(&b.key),
    }
}

fn by_end(a: Option<&Node>, b: Option<&Node>) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => by_name(a, b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn node_line(node: &Node) -> String {
    format!("{} {}", node.kind, name(node))
}

fn edge_line(edge: &Edge) -> String {
    let label = match &edge.label {
        Some(label) => format!(" : \"{}\"", shown_label(label)),
        None => String::new(),
    };
    format!(
        "{} {} → {}{label}",
        edge.kind,
        end(edge.from.as_ref()),
        end(edge.to.as_ref())
    )
}

fn end(node: Option<&Node>) -> String {
    node.map_or("(unconnected)".to_string(), name)
}

fn name(node: &Node) -> String {
    match &node.label {
        Some(label) => format!("\"{}\"", shown_label(label)),
        None => format!("#{}", node.key),
    }
}

fn shown_label(label: &str) -> String {
    let escaped = label.replace('\\', "\\\\").replace('"', "\\\"");
    plain(&escaped.replace("\r\n", "\n"))
}

// All output leaves through here: one Change per line, no escape codes, no reversed text.
fn plain_lines(lines: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    let mut text = String::new();
    for line in lines {
        text.push_str(&plain(line.as_ref()));
        text.push('\n');
    }
    text
}

fn plain(text: &str) -> String {
    text.replace(
        |c: char| c.is_control() || breaks_line(c) || sets_text_direction(c),
        " ",
    )
}

/// Unicode's line and paragraph separators.
fn breaks_line(c: char) -> bool {
    matches!(c, '\u{2028}' | '\u{2029}')
}

/// Unicode's `Bidi_Control` characters.
fn sets_text_direction(c: char) -> bool {
    matches!(c, '\u{061C}' | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}
