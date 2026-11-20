//! Wrap archives must match the `source_hash` / `patch_hash` the wrap pins
//! before anything is extracted (upstream Meson `wrap.py` `check_hash`).

use std::path::{Path, PathBuf};
use std::process::Command;

static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn sha256(path: &Path) -> String {
    let out = Command::new("shasum").args(["-a", "256"]).arg(path).output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap().split_whitespace().next().unwrap().to_string()
}

fn tar(archive: &Path, root: &Path, member: &str) {
    let status = Command::new("tar").arg("czf").arg(archive).arg("-C").arg(root).arg(member).status().unwrap();
    assert!(status.success());
}

struct Fixture {
    dir: PathBuf,
    source: PathBuf,
    patch: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "gluon-wrap-hash-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let staging = dir.join("staging");
        std::fs::create_dir_all(staging.join("foo-1.0")).unwrap();
        std::fs::write(staging.join("foo-1.0/meson.build"), "project('foo')\n").unwrap();
        // The patch archive extracts next to the source tree and adds a file.
        std::fs::create_dir_all(staging.join("patch/foo-1.0")).unwrap();
        std::fs::write(staging.join("patch/foo-1.0/patched.txt"), "patched\n").unwrap();
        std::fs::create_dir_all(dir.join("subprojects/packagecache")).unwrap();
        std::fs::create_dir_all(dir.join("served")).unwrap();
        std::fs::write(dir.join("meson.build"), "project('test')\nsubproject('foo')\n").unwrap();
        let source = dir.join("served/foo-1.0.tar.gz");
        let patch = dir.join("served/foo-patch.tar.gz");
        tar(&source, &staging, "foo-1.0");
        tar(&patch, &staging.join("patch"), "foo-1.0");
        Fixture { dir, source, patch }
    }

    fn write_wrap(&self, body: &str) {
        std::fs::write(self.dir.join("subprojects/foo.wrap"), format!("[wrap-file]\ndirectory = foo-1.0\n{body}")).unwrap();
    }

    /// Put a served archive into packagecache/ the way a Meson download cache holds it.
    fn cache(&self, archive: &Path) {
        std::fs::copy(archive, self.dir.join("subprojects/packagecache").join(archive.file_name().unwrap())).unwrap();
    }

    fn url(&self, archive: &Path) -> String {
        format!("file://{}", archive.display())
    }

    fn setup(&self, extra: &[&str]) -> std::process::Output {
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "setup_child", "--nocapture"])
            .env("GLUON_TEST_SOURCE", &self.dir)
            .env("GLUON_TEST_ARGS", extra.join("\n"))
            .output()
            .unwrap()
    }

    fn extracted(&self) -> bool {
        self.dir.join("subprojects/foo-1.0").exists() || self.dir.join("subprojects/foo").exists()
    }

    fn reject(&self, extra: &[&str], expected: &[&str]) {
        let output = self.setup(extra);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "setup succeeded: {stderr}");
        for part in expected {
            assert!(stderr.contains(part), "missing {part:?} in {stderr}");
        }
        assert!(!self.extracted(), "archive was extracted despite the failure: {stderr}");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn setup_child() {
    if let Some(dir) = std::env::var_os("GLUON_TEST_SOURCE") {
        let dir = PathBuf::from(dir);
        let mut args = vec![
            "meson".to_string(), "setup".to_string(),
            dir.join("build").to_string_lossy().into_owned(),
            dir.to_string_lossy().into_owned(),
        ];
        let extra = std::env::var("GLUON_TEST_ARGS").unwrap_or_default();
        args.extend(extra.split('\n').filter(|a| !a.is_empty()).map(str::to_string));
        std::process::exit(meson::Cli::parse_args_from(args).run());
    }
}

const WRONG: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[test]
fn cached_source_with_wrong_hash_is_rejected_before_extraction() {
    let f = Fixture::new();
    f.cache(&f.source);
    f.write_wrap(&format!(
        "source_url = https://example.invalid/foo-1.0.tar.gz\nsource_filename = foo-1.0.tar.gz\nsource_hash = {WRONG}\n"
    ));
    let actual = sha256(&f.source);
    f.reject(&[], &["foo.wrap", "source_hash", WRONG, &actual]);
}

#[test]
fn cached_patch_with_wrong_hash_is_rejected_before_extraction() {
    let f = Fixture::new();
    f.cache(&f.source);
    f.cache(&f.patch);
    f.write_wrap(&format!(
        "source_url = https://example.invalid/foo-1.0.tar.gz\nsource_filename = foo-1.0.tar.gz\nsource_hash = {}\n\
         patch_url = https://example.invalid/foo-patch.tar.gz\npatch_filename = foo-patch.tar.gz\npatch_hash = {WRONG}\n",
        sha256(&f.source)
    ));
    let actual = sha256(&f.patch);
    f.reject(&[], &["foo.wrap", "patch_hash", WRONG, &actual]);
}

#[test]
fn downloaded_source_with_wrong_hash_is_rejected_and_not_kept() {
    let f = Fixture::new();
    f.write_wrap(&format!(
        "source_url = {}\nsource_filename = foo-1.0.tar.gz\nsource_hash = {WRONG}\n",
        f.url(&f.source)
    ));
    f.reject(&[], &["source_hash", WRONG]);
    assert!(!f.dir.join("subprojects/foo-1.0.tar.gz").exists(), "unverified download left behind");
}

#[test]
fn source_url_without_source_hash_fails_closed() {
    let f = Fixture::new();
    f.cache(&f.source);
    f.write_wrap("source_url = https://example.invalid/foo-1.0.tar.gz\nsource_filename = foo-1.0.tar.gz\n");
    f.reject(&[], &["Missing key 'source_hash'", "foo.wrap"]);
}

#[test]
fn patch_url_without_patch_hash_fails_closed() {
    let f = Fixture::new();
    f.cache(&f.source);
    f.cache(&f.patch);
    f.write_wrap(&format!(
        "source_url = https://example.invalid/foo-1.0.tar.gz\nsource_filename = foo-1.0.tar.gz\nsource_hash = {}\n\
         patch_url = https://example.invalid/foo-patch.tar.gz\npatch_filename = foo-patch.tar.gz\n",
        sha256(&f.source)
    ));
    f.reject(&[], &["Missing key 'patch_hash'"]);
}

#[test]
fn nodownload_refuses_the_network_but_still_uses_verified_cache() {
    let f = Fixture::new();
    f.write_wrap(&format!(
        "source_url = {}\nsource_filename = foo-1.0.tar.gz\nsource_hash = {}\n",
        f.url(&f.source),
        sha256(&f.source)
    ));
    f.reject(&["-Dwrap_mode=nodownload"], &["nodownload"]);
    f.cache(&f.source);
    let output = f.setup(&["--wrap-mode=nodownload"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(f.dir.join("subprojects/foo-1.0/meson.build").exists());
}

#[test]
fn correctly_pinned_cached_source_and_patch_configure() {
    let f = Fixture::new();
    f.cache(&f.source);
    f.cache(&f.patch);
    f.write_wrap(&format!(
        "source_url = https://example.invalid/foo-1.0.tar.gz\nsource_filename = foo-1.0.tar.gz\nsource_hash = {}\n\
         patch_url = https://example.invalid/foo-patch.tar.gz\npatch_filename = foo-patch.tar.gz\npatch_hash = {}\n",
        sha256(&f.source),
        sha256(&f.patch)
    ));
    let output = f.setup(&[]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(f.dir.join("subprojects/foo-1.0/meson.build").exists());
    assert!(f.dir.join("subprojects/foo-1.0/patched.txt").exists());
    assert!(f.dir.join("build/build.ninja").exists());
}

#[test]
fn correctly_pinned_download_configures() {
    let f = Fixture::new();
    f.write_wrap(&format!(
        "source_url = {}\nsource_filename = foo-1.0.tar.gz\nsource_hash = {}\n",
        f.url(&f.source),
        sha256(&f.source).to_uppercase()
    ));
    let output = f.setup(&[]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(f.dir.join("subprojects/foo-1.0/meson.build").exists());
}
