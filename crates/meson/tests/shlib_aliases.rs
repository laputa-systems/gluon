use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn setup(source: &str) -> (std::process::Output, PathBuf) {
    let dir = std::env::temp_dir().join(format!(
        "gluon-shlib-aliases-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    std::fs::write(dir.join("a.c"), "int f(void){return 0;}\n").unwrap();
    std::fs::write(dir.join("sub/b.c"), "int g(void){return 0;}\n").unwrap();
    std::fs::write(
        dir.join("sub/meson.build"),
        "shared_library('inner', 'b.c', version: '2.0.1', soversion: '2')\n",
    )
    .unwrap();
    std::fs::write(dir.join("meson.build"), format!("project('aliases', 'c')\n{source}\nsubdir('sub')\n")).unwrap();
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
            "meson".into(),
            "setup".into(),
            dir.join("build").to_string_lossy().into_owned(),
            dir.to_string_lossy().into_owned(),
        ])
        .run();
        std::process::exit(code);
    }
}

fn link_target(dir: &Path, name: &str) -> Option<String> {
    let path = dir.join(name);
    let meta = std::fs::symlink_metadata(&path).ok()?;
    if !meta.file_type().is_symlink() {
        return None;
    }
    Some(
        std::fs::read_link(&path)
            .unwrap()
            .to_string_lossy()
            .into_owned(),
    )
}

fn expect_link(dir: &Path, name: &str, target: &str) {
    assert_eq!(
        link_target(dir, name).as_deref(),
        Some(target),
        "expected {name} -> {target}"
    );
}

fn expect_no_link(dir: &Path, name: &str) {
    assert_eq!(link_target(dir, name), None, "unexpected symlink {name}");
}

#[test]
fn versioned_shared_library_aliases() {
    let source = "\
shared_library('foo', 'a.c', version: '0.9.2', soversion: '0')
shared_library('ver', 'a.c', version: '1.2.3')
shared_library('so', 'a.c', soversion: '7')
shared_library('plain', 'a.c')
";
    let (output, dir) = setup(source);
    assert!(
        output.status.success(),
        "setup failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let build = dir.join("build");

    if cfg!(target_os = "macos") {
        expect_link(&build, "libfoo.dylib", "libfoo.0.dylib");
        expect_link(&build, "libver.dylib", "libver.1.dylib");
        expect_link(&build, "libso.dylib", "libso.7.dylib");
        expect_no_link(&build, "libfoo.0.dylib");
        expect_no_link(&build, "libfoo.so.0");
    } else {
        expect_link(&build, "libfoo.so.0", "libfoo.so.0.9.2");
        expect_link(&build, "libfoo.so", "libfoo.so.0");
        expect_link(&build, "libver.so.1", "libver.so.1.2.3");
        expect_link(&build, "libver.so", "libver.so.1");
        expect_link(&build, "libso.so", "libso.so.7");
        expect_no_link(&build, "libfoo.so.0.9.2");
        expect_no_link(&build, "libver.so.1.2.3");
    }
    expect_no_link(&build, if cfg!(target_os = "macos") { "libplain.dylib" } else { "libplain.so" });

    let sub = build.join("sub");
    if cfg!(target_os = "macos") {
        expect_link(&sub, "libinner.dylib", "libinner.2.dylib");
    } else {
        expect_link(&sub, "libinner.so.2", "libinner.so.2.0.1");
        expect_link(&sub, "libinner.so", "libinner.so.2");
    }

    std::fs::remove_dir_all(dir).unwrap();
}
