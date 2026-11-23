use std::fs;
use std::process::Command;

#[test]
#[ignore]
fn setup_child() {
    let dir = std::env::var("COMPDB_SOURCE").unwrap();
    let mut args = vec!["meson".to_owned(), "setup".to_owned()];
    if std::env::var_os("COMPDB_RECONFIGURE").is_some() { args.push("--reconfigure".to_owned()); }
    if let Ok(options) = std::env::var("COMPDB_OPTIONS") {
        args.extend(options.split_whitespace().map(str::to_owned));
    }
    args.push(format!("{dir}/build"));
    args.push(dir);
    std::process::exit(meson::Cli::parse_args_from(args).run());
}

#[test]
fn assertion_arguments_follow_ndebug_and_buildtype() {
    let root = std::env::temp_dir().join(format!("gluon-meson-assertions-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    for (index, options, disabled) in [(0, "", false), (1, "-Db_ndebug=true", true), (2, "-Dbuildtype=release -Db_ndebug=if-release", true), (3, "-Dbuildtype=debug -Db_ndebug=if-release", false), (4, "-Dbuildtype=plain -Db_ndebug=if-release", true)] {
        let dir = root.join(index.to_string());
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("meson.build"), "project('test', 'c', 'cpp')\nexecutable('app', 'main.c', 'part.cpp')\n").unwrap();
        fs::write(dir.join("main.c"), "int main(void) { return 0; }\n").unwrap();
        fs::write(dir.join("part.cpp"), "int part() { return 0; }\n").unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "setup_child", "--ignored", "--nocapture"])
            .env("COMPDB_SOURCE", &dir).env("COMPDB_OPTIONS", options).output().unwrap();
        assert!(output.status.success(), "{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        let json = fs::read_to_string(dir.join("build/compile_commands.json")).unwrap();
        let c = json.lines().find(|line| line.contains("\"command\"") && line.contains("main.c.o")).unwrap();
        let cpp = json.lines().find(|line| line.contains("\"command\"") && line.contains("part.cpp.o")).unwrap();
        for line in [c, cpp] { assert_eq!(line.contains("-DNDEBUG"), disabled, "{options}: {line}"); }
        assert!(!c.contains("_GLIBCXX_ASSERTIONS") && !c.contains("_LIBCPP_HARDENING_MODE"), "{c}");
        if !disabled { assert!(cpp.contains("-D_GLIBCXX_ASSERTIONS=1") || cpp.contains("-D_LIBCPP_HARDENING_MODE=_LIBCPP_HARDENING_MODE_FAST"), "{cpp}"); }
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn setup_and_reconfigure_write_only_compile_commands() {
    let dir = std::env::temp_dir().join(format!("gluon-meson-compdb-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("meson.build"), "project('test', 'c', 'cpp')\nexecutable('app', 'main.c', 'part.cpp')\n").unwrap();
    fs::write(dir.join("main.c"), "int main(void) { return 0; }\n").unwrap();
    fs::write(dir.join("part.cpp"), "int part() { return 0; }\n").unwrap();
    for reconfigure in [false, true] {
        if reconfigure { fs::write(dir.join("build/compile_commands.json"), "stale").unwrap(); }
        let mut child = Command::new(std::env::current_exe().unwrap());
        child.args(["--exact", "setup_child", "--ignored", "--nocapture"]).env("COMPDB_SOURCE", &dir);
        if reconfigure { child.env("COMPDB_RECONFIGURE", "1"); }
        let output = child.output().unwrap();
        assert!(output.status.success(), "{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        let json = fs::read_to_string(dir.join("build/compile_commands.json")).unwrap();
        let entries: Vec<deser_value::Value> = deser_json::from_str(&json).unwrap();
        assert_eq!(entries.len(), 2, "{json}");
        assert!(json.contains("main.c") && json.contains("part.cpp"), "{json}");
        assert!(json.contains("main.c.o") && json.contains("part.cpp.o"), "{json}");
        assert!(!json.contains("-o app "), "{json}");
        assert!(json.contains(&dir.join("build").display().to_string()), "{json}");
    }
    fs::remove_dir_all(dir).unwrap();
}
