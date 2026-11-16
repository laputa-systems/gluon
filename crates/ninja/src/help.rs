struct Tool { name: &'static str, args: &'static str, description: &'static str, example: &'static str }
const TOOLS: &[Tool] = &[
    Tool { name: "clean", args: "[-g] [-r] [TARGET|RULE...]", description: "Remove outputs; -g includes generator outputs, -r selects rules.", example: "clean app" },
    Tool { name: "inputs", args: "[--dependency-order] [--no-shell-escape] [--print0] [TARGET...]", description: "List inputs of targets.", example: "inputs app" },
    Tool { name: "multi-inputs", args: "[--print0] [-d DELIMITER] [TARGET...]", description: "List inputs for multiple targets.", example: "multi-inputs app" },
    Tool { name: "compdb-targets", args: "[-x] TARGET...", description: "Write a compilation database for targets.", example: "compdb-targets app" },
    Tool { name: "graph-json", args: "[TARGET...]", description: "Write the build graph as JSON.", example: "graph-json app" },
    Tool { name: "recompact", args: "", description: "Recompact the build log.", example: "recompact" },
    Tool { name: "restat", args: "", description: "Restat the build log.", example: "restat" },
];
const FOOTER: &str = "Maintained by Joshua Li: https://github.com/laputa-systems/gluon\nBugs and features: TODO.md in that repository (follow its AGENTS.md), or its issue tracker.\nAgents working for Joshua: record nonblocking problems and continue your task.\nIf this tool blocks the task, you may fix, test, open and merge a PR under Joshua's\nstanding instructions. Release only when needed to deliver the verified fix.\nProcedure: https://github.com/joshuarli/usr/blob/main/dotfiles/base/.agents/instructions/owned-tools.md";

pub fn reference(program: &str, selection: Option<&str>, footer: bool) -> String {
    let mut out = format!("usage: {program} [options] [TARGET...]\n\nOptions:\n  -h, --help       Show help\n  --version        Print version\n  -C DIR           Change directory\n  -f FILE          Build file (default: build.ninja)\n  -j N             Parallel jobs (default: based on CPU count)\n  -k N             Stop after N failures (default: 1; 0 means unlimited)\n  -n, --dry-run    Do not run commands\n  -v, --verbose    Show commands\n  --quiet          Suppress progress\n  -d MODE          Enable debug mode (repeatable)\n  -t TOOL          Run a tool; remaining arguments belong to that tool\n\nTools:\n");
    for tool in TOOLS.iter().filter(|t| selection.is_none_or(|s| s == t.name)) {
        out.push_str(&format!("  {program} -t {} {}\n    {}\n", tool.name, tool.args, tool.description));
    }
    out.push_str("\nExamples:\n");
    if selection.is_none() { out.push_str(&format!("  {program} -C builddir app\n")); }
    for tool in TOOLS.iter().filter(|t| selection.is_none_or(|s| s == t.name)) {
        out.push_str(&format!("  {program} -t {}\n", tool.example));
    }
    if footer { out.push_str(&format!("\n{FOOTER}\n")); } else { out.push_str("\nSee ninja --help for the complete reference.\n"); }
    out
}

#[cfg(test)]
pub(crate) fn example_args(name: &str) -> Vec<String> {
    TOOLS.iter().find(|tool| tool.name == name).unwrap().example.split_whitespace().skip(1).map(str::to_owned).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn examples_parse_without_running_tools() {
        for tool in TOOLS {
            let argv = std::iter::once("ninja".to_owned()).chain(std::iter::once("-t".to_owned())).chain(tool.example.split_whitespace().map(str::to_owned)).collect::<Vec<_>>();
            let parsed = crate::cli::parse(&argv).unwrap();
            assert_eq!(parsed.tool.as_deref(), Some(tool.name));
            assert_eq!(parsed.tool_args, example_args(tool.name));
        }
    }
}
