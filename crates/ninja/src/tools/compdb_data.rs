use crate::graph::{Edge, State};
use crate::manifest::expand;

fn shell_path(path: &str) -> String {
    if path.bytes().all(|c| c.is_ascii_alphanumeric() || b"_+-./".contains(&c)) {
        path.to_owned()
    } else {
        format!("'{}'", path.replace('\'', "'\\''"))
    }
}

fn expand_edge(state: &State, edge: &Edge, value: &str) -> String {
    let rule = state.rules.get(&edge.rule);
    let inputs = edge.inputs.iter().map(|p| shell_path(p)).collect::<Vec<_>>();
    let outputs = edge.outputs.iter().map(|p| shell_path(p)).collect::<Vec<_>>();
    expand(value, &|name| match name {
        "in" => Some(inputs.join(" ")),
        "in_newline" => Some(inputs.join("\n")),
        "out" => Some(outputs.join(" ")),
        _ => edge.bindings.get(name).cloned().or_else(|| {
            rule.and_then(|r| r.bindings.get(name))
                .map(|v| expand(v, &|n| state.bindings.get(n).cloned()))
        }).or_else(|| state.bindings.get(name).cloned()),
    })
}

fn binding(state: &State, edge: &Edge, name: &str) -> Option<String> {
    edge.bindings.get(name).cloned().or_else(|| {
        state.rules.get(&edge.rule)?.bindings.get(name)
            .map(|value| expand_edge(state, edge, value))
    })
}

fn command(state: &State, edge: &Edge, expand_rsp: bool) -> String {
    let mut command = binding(state, edge, "command").unwrap_or_default();
    if expand_rsp && let Some(rsp) = binding(state, edge, "rspfile") && !rsp.is_empty()
        && let Some(index) = command.find(&rsp)
    {
        let prefix = if index > 0 && command.as_bytes()[index - 1] == b'@' {
            1
        } else if index >= 3 && command.as_bytes().get(index - 3..index) == Some(b"-f ") {
            3
        } else if index >= 14 && command.as_bytes().get(index - 14..index) == Some(b"--option-file=") {
            14
        } else {
            0
        };
        if prefix != 0 {
            let content = binding(state, edge, "rspfile_content").unwrap_or_default().replace('\n', " ");
            command.replace_range(index - prefix..index + rsp.len(), &content);
        }
    }
    command
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

pub(crate) fn render(state: &State, directory: &str, edges: impl Iterator<Item = usize>, expand_rsp: bool) -> String {
    let mut out = String::from("[\n");
    let mut first = true;
    for index in edges {
        let edge = &state.edges[index];
        if !first { out.push_str(",\n"); }
        first = false;
        out.push_str(&format!("  {{\n    \"directory\": \"{}\",\n    \"command\": \"{}\",\n    \"file\": \"{}\",\n    \"output\": \"{}\"\n  }}",
            json_escape(directory), json_escape(&command(state, edge, expand_rsp)),
            json_escape(edge.inputs.first().map_or("", String::as_str)),
            json_escape(edge.outputs.first().map_or("", String::as_str))));
    }
    if !first { out.push('\n'); }
    out.push_str("]\n");
    out
}

pub(crate) fn render_rules(state: &State, directory: &str, rules: &[&str], expand_rsp: bool) -> String {
    let edges = state.edges.iter().enumerate().filter_map(|(index, edge)| {
        let rule = state.rules.get(&edge.rule);
        let has_command = edge.bindings.contains_key("command") || rule.is_some_and(|r| r.bindings.contains_key("command"));
        (has_command && (rules.is_empty() || rules.iter().any(|name| *name == rule.map_or(edge.rule.as_str(), |r| r.name.as_str())))).then_some(index)
    });
    render(state, directory, edges, expand_rsp)
}
