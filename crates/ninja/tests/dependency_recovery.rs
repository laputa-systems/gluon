use std::fs::{self, File, FileTimes};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Output};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[test]
#[ignore]
fn ninja_child() {
    let args: Vec<String> = std::env::var("NINJA_TEST_ARGS").unwrap().split_whitespace().map(str::to_owned).collect();
    let mut argv = vec!["ninja".to_owned()];
    argv.extend(args);
    if ninja::main_from(argv) != ExitCode::SUCCESS { std::process::exit(1); }
}

fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gluon-recovery-{}-{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("build.ninja"), "rule copy\n  command = cp -p header app && printf 'app: header\\n' > app.d\n  depfile = app.d\n  deps = gcc\n  restat = 1\nbuild app: copy\n").unwrap();
    source(&dir, "first", 1_700_000_000);
    dir
}

fn source(dir: &Path, contents: &str, seconds: u64) {
    let path = dir.join("header");
    fs::write(&path, contents).unwrap();
    File::open(path).unwrap().set_times(FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(seconds))).unwrap();
}

fn run(dir: &Path, args: &str) -> Output {
    Command::new(std::env::current_exe().unwrap()).args(["--exact", "ninja_child", "--ignored", "--nocapture"]).env("NINJA_TEST_ARGS", args).env_remove("NINJA_STATUS").current_dir(dir).output().unwrap()
}

fn build(dir: &Path) {
    let result = run(dir, "");
    assert!(result.status.success(), "{result:?}");
}

#[test]
fn rolled_back_dependency_record_rebuilds_output() {
    let dir = scratch();
    build(&dir);
    let old_log = fs::read(dir.join(".ninja_deps")).unwrap();
    source(&dir, "second", 1_700_000_001);
    build(&dir);
    fs::write(dir.join(".ninja_deps"), old_log).unwrap();
    source(&dir, "third", 1_700_000_001);
    build(&dir);
    assert_eq!(fs::read_to_string(dir.join("app")).unwrap(), "third");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn pending_depfile_forces_retry() {
    let dir = scratch();
    build(&dir);
    fs::write(dir.join("app.d"), "app: header\n").unwrap();
    source(&dir, "second", 1_700_000_000);
    build(&dir);
    assert_eq!(fs::read_to_string(dir.join("app")).unwrap(), "second");
    assert!(!dir.join("app.d").exists());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn dependency_log_failure_keeps_depfile_and_drains_commands() {
    let dir = scratch();
    fs::write(dir.join("build.ninja"), "rule copy\n  command = while [ ! -f started ]; do sleep 0.01; done; cp -p header app && printf 'app: header\\n' > app.d\n  depfile = app.d\n  deps = gcc\nrule slow\n  command = touch started; sleep 0.3; touch finished\nrule finish\n  command = cp $in $out\nbuild app: copy\nbuild done: finish app\nbuild finished: slow\ndefault done finished\n").unwrap();
    fs::create_dir(dir.join(".ninja_deps")).unwrap();
    let failed = run(&dir, "-j2 -k0");
    assert!(!failed.status.success(), "{failed:?}");
    assert!(String::from_utf8_lossy(&failed.stderr).contains(".ninja_deps"), "{failed:?}");
    assert_eq!(fs::read_to_string(dir.join("app.d")).unwrap(), "app: header\n");
    assert!(!dir.join("done").exists());
    assert!(dir.join("finished").exists());
    fs::remove_dir(dir.join(".ninja_deps")).unwrap();
    build(&dir);
    assert_eq!(fs::read_to_string(dir.join("done")).unwrap(), "first");
    assert!(!dir.join("app.d").exists());
    fs::remove_dir_all(dir).unwrap();
}
