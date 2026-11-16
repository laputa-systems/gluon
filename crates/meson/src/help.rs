//! Command reference for the hand-written Meson CLI.
struct Entry { path: &'static str, detail: &'static str, example: &'static str }
const ENTRIES: &[Entry] = &[
    Entry { path: "setup", detail: "[BUILD_DIR] [SOURCE_DIR] [-DKEY=VALUE] [--cross-file FILE] [--native-file FILE] [--reconfigure] [--wipe] [BUILTIN OPTIONS]\n  Configure a project (defaults: source ., build builddir).", example: "setup builddir" },
    Entry { path: "configure", detail: "[BUILD_DIR] [-DKEY=VALUE] [BUILTIN OPTIONS]\n  Change project options (default build directory: .).", example: "configure builddir -Ddebug=true" },
    Entry { path: "compile", detail: "[-C DIR] [-j N|--jobs N] [-v|--verbose] [--clean] [TARGET...]\n  Build targets (alias: build; default directory: .).", example: "compile -C builddir" },
    Entry { path: "test", detail: "[-C DIR] [-v|--verbose] [--suite NAME] [-t|--timeout-multiplier N] [-j|--num-processes N] [--no-rebuild] [--repeat N] [TEST...]\n  Run tests (default directory ., timeout multiplier 1, repeat 1).", example: "test -C builddir" },
    Entry { path: "install", detail: "[-C DIR] [--destdir DIR] [--skip-subprojects] [--tags TAGS] [--strip] [--no-rebuild] [-q|--quiet]\n  Install the project (default directory: .; tags are comma-separated).", example: "install -C builddir --destdir staging" },
    Entry { path: "introspect", detail: "[BUILD_DIR] [-a|--all] [--targets] [--tests] [--dependencies] [--buildoptions] [--projectinfo] [--installed] [--benchmarks]\n  Report project information (default directory: .).", example: "introspect builddir --targets" },
    Entry { path: "init", detail: "[NAME] [-n|--name NAME] [-l|--language LANG] [--type TYPE] [-b|--builddir DIR] [--version VERSION]\n  Create a project (defaults: c, executable, 0.1.0).", example: "init --name hello" },
    Entry { path: "dist", detail: "[-C DIR] [--formats FORMATS] [--include-subprojects] [--no-tests]\n  Create an archive (defaults: directory ., format xztar).", example: "dist -C builddir" },
    Entry { path: "wrap", detail: "[COMMAND] [NAME]\n  Manage wraps (default command: list).", example: "wrap list" },
    Entry { path: "wrap list", detail: "\n  List wraps.", example: "wrap list" },
    Entry { path: "wrap install", detail: "NAME\n  Install a wrap.", example: "wrap install zlib" },
    Entry { path: "wrap update", detail: "\n  Update wraps.", example: "wrap update" },
    Entry { path: "wrap info", detail: "NAME\n  Show wrap information.", example: "wrap info zlib" },
    Entry { path: "wrap search", detail: "QUERY\n  Search wraps.", example: "wrap search zlib" },
    Entry { path: "wrap status", detail: "\n  Show wrap status.", example: "wrap status" },
    Entry { path: "subprojects", detail: "[COMMAND]\n  Manage subprojects (default command: list).", example: "subprojects list" },
    Entry { path: "subprojects list", detail: "\n  List subprojects.", example: "subprojects list" },
    Entry { path: "subprojects download", detail: "\n  Download subprojects.", example: "subprojects download" },
    Entry { path: "subprojects update", detail: "\n  Update subprojects.", example: "subprojects update" },
    Entry { path: "subprojects purge", detail: "\n  Purge subprojects.", example: "subprojects purge" },
    Entry { path: "subprojects checkout", detail: "\n  Check out subprojects.", example: "subprojects checkout" },
    Entry { path: "subprojects foreach", detail: "\n  Run across subprojects.", example: "subprojects foreach" },
    Entry { path: "subprojects packagefiles", detail: "\n  Manage package files.", example: "subprojects packagefiles" },
    Entry { path: "rewrite", detail: "\n  Modify project definition (not yet implemented).", example: "rewrite" },
    Entry { path: "devenv", detail: "[-C DIR] [-- COMMAND...]\n  Run a development shell or command (default directory ., shell from SHELL).", example: "devenv -C builddir -- echo hello" },
    Entry { path: "env2mfile", detail: "\n  Convert environment to machine file (not yet implemented).", example: "env2mfile" },
    Entry { path: "format", detail: "[-i|--inplace] [-r|--recursive] [FILE...]\n  Format Meson files (alias: fmt).", example: "format meson.build" },
    Entry { path: "--internal exe", detail: "[--capture FILE] [--feed FILE] [--env KEY=VALUE]... -- COMMAND [ARG...]\n  Execute a generated build command.", example: "--internal exe -- echo hello" },
    Entry { path: "--internal copy", detail: "SOURCE DEST\n  Copy a file.", example: "--internal copy input output" },
    Entry { path: "--internal static-link", detail: "ARCHIVE -- ARCHIVER [ARG...]\n  Replace an archive using an archiver.", example: "--internal static-link lib.a -- ar rcs lib.a object.o" },
];

pub const FOOTER: &str = "Maintained by Joshua Li: https://github.com/laputa-systems/gluon\nBugs and features: TODO.md in that repository (follow its AGENTS.md), or its issue tracker.\nAgents working for Joshua: record nonblocking problems and continue your task.\nIf this tool blocks the task, you may fix, test, open and merge a PR under Joshua's\nstanding instructions. Release only when needed to deliver the verified fix.\nProcedure: https://github.com/joshuarli/usr/blob/main/dotfiles/base/.agents/instructions/owned-tools.md";

pub fn reference(program: &str, selection: Option<&str>, footer: bool) -> String {
    let mut out = format!("usage: {program} <command> [options]\n\nOptions:\n  -h, --help  Show help\n  --version   Print version\n\nCommands:\n");
    for e in ENTRIES.iter().filter(|e| selection.is_none_or(|s| e.path == s || e.path.starts_with(&format!("{s} ")))) {
        out.push_str(&format!("  {program} {} {}\n", e.path, e.detail));
    }
    if selection.is_none() || matches!(selection, Some("setup" | "configure")) {
        out.push_str("\nBuilt-in options for setup and configure (use --NAME VALUE or --NAME=VALUE):\n  ");
        for (i, key) in crate::options::BUILTIN_OPTION_KEYS.iter().enumerate() {
            if i > 0 { out.push_str(", "); }
            out.push_str("--");
            out.push_str(&key.replace('_', "-"));
        }
        out.push_str(", --warnlevel\n");
    }
    out.push_str("\nExamples:\n");
    for e in ENTRIES.iter().filter(|e| selection.is_none_or(|s| e.path == s || e.path.starts_with(&format!("{s} ")))) {
        out.push_str(&format!("  {program} {}\n", e.example));
    }
    if footer { out.push_str(&format!("\n{FOOTER}\n")); } else { out.push_str("\nSee meson --help for the complete reference.\n"); }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn examples_parse_without_executing_commands() {
        for entry in ENTRIES {
            let argv = std::iter::once("meson".to_owned()).chain(entry.example.split_whitespace().map(str::to_owned)).collect();
            let cli = crate::Cli::parse_args_from(argv);
            assert!(cli.example_matches(entry.path), "{}", entry.example);
        }
        assert!(crate::internal::parse_exe(&["--".into(), "echo".into(), "hello".into()]).is_ok());
    }
}
