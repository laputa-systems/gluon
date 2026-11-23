use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

#[test]
fn cli_child() {
    if let Some(args) = std::env::var_os("GLUON_PRIVATE_INCLUDE_ARGS") {
        let args = args.to_str().unwrap().split('\n').map(str::to_owned).collect();
        std::process::exit(meson::Cli::parse_args_from(args).run());
    }
}

fn setup(dir: &Path) {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "cli_child", "--nocapture"])
        .current_dir(dir)
        .env("GLUON_PRIVATE_INCLUDE_ARGS", "meson\nsetup\nbuild\n.")
        .output().unwrap();
    assert!(output.status.success(), "{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
}

fn fixture() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("gluon-private-include-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&dir).unwrap();
    dir.canonicalize().unwrap()
}

fn include_args(dir: &Path, target: &str) -> Vec<String> {
    let manifest = std::fs::read_to_string(dir.join("build/build.ninja")).unwrap();
    let edge = manifest.lines().skip_while(|line| !line.starts_with(&format!("build {target}.p/main.c.o:")))
        .find(|line| line.trim_start().starts_with("ARGS = ")).unwrap();
    edge.split_whitespace().filter(|arg| arg.starts_with("-I")).map(str::to_owned).collect()
}

#[test]
fn private_include_precedes_implicit_directories() {
    let dir = fixture();
    std::fs::write(dir.join("meson.build"), "project('p', 'c')\nexecutable('app', 'main.c')\nexecutable('noimplicit', 'main.c', implicit_include_directories: false)\n").unwrap();
    std::fs::write(dir.join("main.c"), "int main(void) { return 0; }\n").unwrap();
    setup(&dir);
    assert_eq!(include_args(&dir, "app"), vec![format!("-I{}/build/app.p", dir.display()), format!("-I{}/build", dir.display()), format!("-I{}", dir.display())]);
    assert_eq!(include_args(&dir, "noimplicit"), vec![format!("-I{}/build/noimplicit.p", dir.display())]);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn generated_private_header_takes_precedence() {
    for implicit in [true, false] {
        let dir = fixture();
        std::fs::write(dir.join("meson.build"), format!("project('p', 'c')\ngen = custom_target('gen', output: 'gen.h', command: [find_program('python3'), files('gen.py'), '@OUTPUT@'])\nexecutable('app', 'main.c', gen, implicit_include_directories: {implicit})\n")).unwrap();
        std::fs::write(dir.join("main.c"), "#include \"gen.h\"\nint main(void) { return VALUE != 7; }\n").unwrap();
        std::fs::write(dir.join("gen.py"), "import pathlib, sys\np = pathlib.Path(sys.argv[1])\np.write_text('#define VALUE 1\\n')\nprivate = p.parent / 'app.p'\nprivate.mkdir(exist_ok=True)\n(private / 'gen.h').write_text('#define VALUE 7\\n')\n").unwrap();
        setup(&dir);
        let output = Command::new("ninja").arg("-C").arg(dir.join("build")).output().unwrap();
        assert!(output.status.success(), "{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        assert!(Command::new(dir.join("build/app")).status().unwrap().success());
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn subdir_targets_include_only_their_own_directories() {
    let dir = fixture();
    std::fs::create_dir(dir.join("sub")).unwrap();
    std::fs::write(dir.join("meson.build"), "project('p', 'c')\nsubdir('sub')\n").unwrap();
    std::fs::write(dir.join("sub/meson.build"), "executable('inner', 'main.c')\n").unwrap();
    std::fs::write(dir.join("sub/main.c"), "int main(void) { return 0; }\n").unwrap();
    setup(&dir);
    assert_eq!(include_args(&dir, "sub/inner"), vec![
        format!("-I{}/build/sub/inner.p", dir.display()),
        format!("-I{}/build/sub", dir.display()),
        format!("-I{}/sub", dir.display()),
    ]);
    std::fs::remove_dir_all(dir).unwrap();
}
