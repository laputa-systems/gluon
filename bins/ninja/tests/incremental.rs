use std::fs::{self, File, FileTimes};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn build(dir: &Path) {
    let output = Command::new(env!("CARGO_BIN_EXE_ninja"))
        .arg("-j1")
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_source(path: &Path, contents: &str, seconds: u64) {
    fs::write(path, contents).unwrap();
    File::open(path)
        .unwrap()
        .set_times(FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(seconds)))
        .unwrap();
}

#[test]
fn ordinary_inputs_rebuild_and_restat_stays_idle_after_observing_them() {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("gluon-incremental-{}-{nonce}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    fs::write(
        dir.join("build.ninja"),
        r"rule copy
  command = cp -p input $out
rule stable
  command = test -f $out || cp -p baseline $out; printf 'run\n' >> stable-runs
  restat = 1
rule downstream
  command = cp -p $in $out && printf 'run\n' >> downstream-runs
build direct: copy input
build implicit: copy | input
build ordered: copy || input
build alias: phony input
build through-alias: copy alias
build stamp: stable restat-input
build result: downstream stamp
default direct implicit ordered through-alias result
",
    )
    .unwrap();
    write_source(&dir.join("input"), "first", 1_700_000_000);
    write_source(&dir.join("baseline"), "unchanged", 1_700_000_000);
    write_source(&dir.join("restat-input"), "first", 1_700_000_000);

    build(&dir);
    write_source(&dir.join("input"), "second", 1_700_000_001);
    std::thread::sleep(Duration::from_millis(25));
    fs::write(dir.join("restat-input"), "second").unwrap();
    build(&dir);
    build(&dir);

    for output in ["direct", "implicit", "through-alias"] {
        assert_eq!(fs::read_to_string(dir.join(output)).unwrap(), "second", "{output}");
    }
    assert_eq!(fs::read_to_string(dir.join("ordered")).unwrap(), "first");
    assert_eq!(fs::read_to_string(dir.join("stable-runs")).unwrap(), "run\nrun\n");
    assert_eq!(fs::read_to_string(dir.join("downstream-runs")).unwrap(), "run\n");
    fs::remove_dir_all(dir).unwrap();
}
