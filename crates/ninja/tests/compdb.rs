use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode};

#[test]
#[ignore]
fn ninja_child() {
    let args = std::env::var("COMPDB_ARGS").unwrap();
    let mut argv = vec!["ninja".to_owned()];
    argv.extend(args.split_whitespace().map(str::to_owned));
    if ninja::main_from(argv) != ExitCode::SUCCESS { std::process::exit(1); }
}

fn run(dir: &Path, args: &str) -> String {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "ninja_child", "--ignored", "--nocapture"])
        .env("COMPDB_ARGS", args).current_dir(dir).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn compdb_selects_rules_expands_responses_and_escapes_json() {
    let dir = std::env::temp_dir().join(format!("gluon-compdb-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("build.ninja"), concat!(
        "rule cc\n  command = cc @args.rsp -DNAME=\"a\\b\"\n  rspfile = args.rsp\n  rspfile_content = $in_newline\n",
        "rule link\n  command = ld $in -o $out\n",
        "build a.o: cc a.c b.c\nbuild app: link a.o\nbuild all: phony app\nbuild x$ y.o: link x$ y.c\n"
    )).unwrap();
    let all = run(&dir, "-t compdb");
    assert_eq!(all.matches("\"directory\"").count(), 3, "{all}");
    assert!(all.contains("cc @args.rsp -DNAME=\\\"a\\\\b\\\""), "{all}");
    assert!(all.contains("\"file\": \"a.c\""), "{all}");
    assert!(all.contains("\"output\": \"a.o\""), "{all}");
    assert!(all.contains("ld a.o -o app"), "{all}");
    assert!(all.contains("ld 'x y.c' -o 'x y.o'"), "{all}");
    let filtered = run(&dir, "-t compdb -x cc");
    assert_eq!(filtered.matches("\"directory\"").count(), 1, "{filtered}");
    assert!(filtered.contains("cc a.c b.c -DNAME=\\\"a\\\\b\\\""), "{filtered}");
    assert!(!filtered.contains("ld a.o"), "{filtered}");
    assert!(!run(&dir, "-t compdb unknown").contains("\"directory\""));
    assert!(run(&dir, "-t compdb-targets -x a.o").contains("cc a.c b.c"));
    fs::remove_dir_all(dir).unwrap();
}
