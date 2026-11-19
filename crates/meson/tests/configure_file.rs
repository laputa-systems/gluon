use std::path::PathBuf;
use std::process::Command;

fn setup(source: &str) -> (std::process::Output, PathBuf) {
    let dir = std::env::temp_dir().join(format!(
        "gluon-configure-file-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("input.in"), "value=@VALUE@\n").unwrap();
    std::fs::write(dir.join("meson.build"), format!("project('test')\n{source}\n")).unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "setup_child", "--nocapture"])
        .env("GLUON_TEST_SOURCE", &dir)
        .output().unwrap();
    (output, dir)
}

#[test]
fn setup_child() {
    if let Some(dir) = std::env::var_os("GLUON_TEST_SOURCE") {
        let dir = PathBuf::from(dir);
        let code = meson::Cli::parse_args_from(vec![
            "meson".into(), "setup".into(), dir.join("build").to_string_lossy().into_owned(),
            dir.to_string_lossy().into_owned(),
        ]).run();
        std::process::exit(code);
    }
}

fn reject(source: &str, expected: &[&str]) {
    let (output, dir) = setup(source);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "setup succeeded: {stderr}");
    for part in expected {
        assert!(stderr.contains(part), "missing {part:?} in {stderr}");
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn missing_configuration_input_fails_setup() {
    reject("configure_file(input: 'missing.in', output: 'out', configuration: configuration_data())", &["missing.in"]);
}

#[test]
fn failed_command_fails_setup() {
    reject("configure_file(output: 'out', command: ['sh', '-c', 'exit 7'])", &["exit 7", "7"]);
}

#[test]
fn command_without_output_fails_setup() {
    reject("configure_file(output: 'out', command: ['sh', '-c', 'exit 0'])", &["out", "output"]);
}

#[test]
fn successful_configuration_preserves_output() {
    let (output, dir) = setup("configure_file(output: 'out', configuration: {'VALUE': 'hello'})");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(std::fs::read_to_string(dir.join("build/out")).unwrap().contains("#define VALUE hello"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn successful_input_and_capture_preserve_bytes() {
    let (output, dir) = setup("configure_file(input: 'input.in', output: 'configured', configuration: {'VALUE': 'hello'})\nconfigure_file(output: 'captured', command: ['sh', '-c', 'printf captured'], capture: true)");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(std::fs::read(dir.join("build/configured")).unwrap(), b"value=hello\n");
    assert_eq!(std::fs::read(dir.join("build/captured")).unwrap(), b"captured");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn missing_command_fails_setup() {
    reject("configure_file(output: 'out', command: ['/no/such/gluon-command'])", &["/no/such/gluon-command"]);
}
