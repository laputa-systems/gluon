use std::fs;
use std::process::Command;

#[test]
#[ignore]
fn setup_child() {
    let dir = std::env::var("COMPDB_SOURCE").unwrap();
    let mut args = vec!["meson".to_owned(), "setup".to_owned()];
    if std::env::var_os("COMPDB_RECONFIGURE").is_some() { args.push("--reconfigure".to_owned()); }
    args.push(format!("{dir}/build"));
    args.push(dir);
    std::process::exit(meson::Cli::parse_args_from(args).run());
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
