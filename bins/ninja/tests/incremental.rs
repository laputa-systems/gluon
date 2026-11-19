use std::fs::{self, File, FileTimes};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ninja"))
        .arg("-j1")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

fn build(dir: &Path, args: &[&str]) {
    let output = run(dir, args);
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn scratch(name: &str) -> PathBuf {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir =
        std::env::temp_dir().join(format!("gluon-{name}-{}-{nonce}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    dir
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
    let dir = scratch("incremental");
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

    build(&dir, &[]);
    write_source(&dir.join("input"), "second", 1_700_000_001);
    std::thread::sleep(Duration::from_millis(25));
    fs::write(dir.join("restat-input"), "second").unwrap();
    build(&dir, &[]);
    build(&dir, &[]);

    for output in ["direct", "implicit", "through-alias"] {
        assert_eq!(fs::read_to_string(dir.join(output)).unwrap(), "second", "{output}");
    }
    assert_eq!(fs::read_to_string(dir.join("ordered")).unwrap(), "first");
    assert_eq!(fs::read_to_string(dir.join("stable-runs")).unwrap(), "run\nrun\n");
    assert_eq!(fs::read_to_string(dir.join("downstream-runs")).unwrap(), "run\n");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn stale_generated_dyndep_cannot_supply_a_missing_manifest_input() {
    let dir = scratch("stale-dyndep");
    let manifest = r"rule write_dd
  command = cp first.dd deps.dd
rule generate
  command = touch primary additional
rule consume
  command = test -f additional && touch $out
build deps.dd: write_dd first.dd new.dd
build primary: generate || deps.dd
  dyndep = deps.dd
build output: consume | additional || deps.dd
  dyndep = deps.dd
default output
";
    let old = "ninja_dyndep_version = 1\nbuild primary | additional: dyndep\nbuild output: dyndep\n";
    let new = "ninja_dyndep_version = 1\nbuild primary: dyndep\nbuild output: dyndep\n";
    fs::write(dir.join("build.ninja"), manifest).unwrap();
    fs::write(dir.join("first.dd"), old).unwrap();
    fs::write(dir.join("new.dd"), new).unwrap();
    build(&dir, &["deps.dd"]);
    build(&dir, &[]);
    fs::remove_file(dir.join("additional")).unwrap();
    fs::remove_file(dir.join("output")).unwrap();

    fs::write(
        dir.join("build.ninja"),
        manifest.replace("cp first.dd", "cp new.dd"),
    )
    .unwrap();
    for flags in [&["-n"][..], &[]] {
        let result = run(&dir, flags);
        assert_eq!(result.status.code(), Some(1), "{result:?}");
        assert_eq!(fs::read_to_string(dir.join("deps.dd")).unwrap(), old);
        assert!(!dir.join("additional").exists());
        assert!(!dir.join("output").exists());
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn dirty_prerequisite_makes_generated_dyndep_unavailable() {
    let dir = scratch("stale-dyndep-prerequisite");
    fs::write(
        dir.join("build.ninja"),
        r"rule copy
  command = cp $in $out
rule generate
  command = touch primary additional
rule consume
  command = test -f additional && touch $out
build input.dd: copy template.dd
build deps.dd: copy input.dd
build primary: generate || deps.dd
  dyndep = deps.dd
build output: consume | additional || deps.dd
  dyndep = deps.dd
default output
",
    )
    .unwrap();
    let old = "ninja_dyndep_version = 1\nbuild primary | additional: dyndep\nbuild output: dyndep\n";
    fs::write(dir.join("template.dd"), old).unwrap();
    build(&dir, &["deps.dd"]);
    build(&dir, &[]);
    fs::remove_file(dir.join("additional")).unwrap();
    fs::remove_file(dir.join("output")).unwrap();
    std::thread::sleep(Duration::from_millis(25));
    fs::write(
        dir.join("template.dd"),
        "ninja_dyndep_version = 1\nbuild primary: dyndep\nbuild output: dyndep\n",
    )
    .unwrap();

    for flags in [&["-n"][..], &[]] {
        let result = run(&dir, flags);
        assert_eq!(result.status.code(), Some(1), "{result:?}");
        assert_eq!(fs::read_to_string(dir.join("input.dd")).unwrap(), old);
        assert_eq!(fs::read_to_string(dir.join("deps.dd")).unwrap(), old);
        assert!(!dir.join("output").exists());
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn nested_dyndep_inputs_participate_in_metadata_readiness() {
    let dir = scratch("nested-dyndep");
    fs::write(
        dir.join("build.ninja"),
        r#"rule copy
  command = cp $in $out
rule select
  command = cp "$$(cat trigger)" $out
rule generate
  command = touch primary additional
rule consume
  command = test -f additional && touch $out
build scan.dd: copy scanner.dd
build trigger: copy trigger-source
build deps.dd: select first.dd new.dd || scan.dd
  dyndep = scan.dd
build primary: generate || deps.dd
  dyndep = deps.dd
build output: consume | additional || deps.dd
  dyndep = deps.dd
default output
"#,
    )
    .unwrap();
    let old = "ninja_dyndep_version = 1\nbuild primary | additional: dyndep\nbuild output: dyndep\n";
    fs::write(dir.join("first.dd"), old).unwrap();
    fs::write(
        dir.join("new.dd"),
        "ninja_dyndep_version = 1\nbuild primary: dyndep\nbuild output: dyndep\n",
    )
    .unwrap();
    fs::write(
        dir.join("scanner.dd"),
        "ninja_dyndep_version = 1\nbuild deps.dd: dyndep | trigger\n",
    )
    .unwrap();
    fs::write(dir.join("trigger-source"), "first.dd").unwrap();
    build(&dir, &["trigger", "deps.dd"]);
    build(&dir, &[]);
    fs::remove_file(dir.join("additional")).unwrap();
    fs::remove_file(dir.join("output")).unwrap();
    std::thread::sleep(Duration::from_millis(25));
    fs::write(dir.join("trigger-source"), "new.dd").unwrap();

    for flags in [&["-n"][..], &[]] {
        let result = run(&dir, flags);
        assert_eq!(result.status.code(), Some(1), "{result:?}");
        assert_eq!(fs::read_to_string(dir.join("trigger")).unwrap(), "first.dd");
        assert_eq!(fs::read_to_string(dir.join("deps.dd")).unwrap(), old);
        assert!(!dir.join("output").exists());
    }
    fs::remove_dir_all(dir).unwrap();
}
