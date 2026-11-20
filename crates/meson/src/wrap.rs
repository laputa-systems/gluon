/// Wrap file handling: download, extract, and patch subproject dependencies.
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Download and extract a wrap dependency.
///
/// `wrap_mode` is the value of the `wrap_mode` builtin option. Under
/// `nodownload` nothing is fetched from the network; archives already present
/// in `packagefiles/` or `packagecache/` are still used (after hash checks).
pub fn download_wrap(wrap_file: &str, dest_dir: &str, wrap_mode: &str) -> Result<(), String> {
    let config = parse_wrap_file(wrap_file)?;
    let wrap_name = Path::new(wrap_file)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| wrap_file.to_string());
    let network = wrap_mode != "nodownload";

    // Determine wrap type from section name ([wrap-file], [wrap-git], etc.)
    // or from explicit "type" key in [wrap] section
    let wrap_type = if config.contains_key("wrap-file") {
        "file"
    } else if config.contains_key("wrap-git") {
        "git"
    } else if config.contains_key("wrap-hg") {
        "hg"
    } else if config.contains_key("wrap-svn") {
        "svn"
    } else {
        config
            .get("wrap")
            .and_then(|s| s.get("type"))
            .map(|s| s.as_str())
            .unwrap_or("file")
    };

    // Normalize: get the wrap section data regardless of section naming style
    let normalized = normalize_wrap_config(&config);

    if !network && wrap_type != "file" {
        return Err(format!(
            "Automatic wrap-based subproject downloading is disabled (wrap_mode=nodownload): {}",
            wrap_name
        ));
    }

    match wrap_type {
        "file" => download_file_wrap(&normalized, dest_dir, &wrap_name, network),
        "git" => download_git_wrap(&normalized, dest_dir),
        "hg" => download_hg_wrap(&normalized, dest_dir),
        "svn" => download_svn_wrap(&normalized, dest_dir),
        _ => Err(format!("Unknown wrap type: {}", wrap_type)),
    }
}

/// Normalize wrap config so that the main section is always accessible as "wrap"
fn normalize_wrap_config(
    config: &HashMap<String, HashMap<String, String>>,
) -> HashMap<String, HashMap<String, String>> {
    let mut normalized = config.clone();
    // If there's a [wrap-file], [wrap-git], etc. section, merge it into "wrap"
    for key in &["wrap-file", "wrap-git", "wrap-hg", "wrap-svn"] {
        if let Some(section) = config.get(*key) {
            let wrap = normalized.entry("wrap".to_string()).or_default();
            for (k, v) in section {
                wrap.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
    }
    normalized
}

fn parse_wrap_file(path: &str) -> Result<HashMap<String, HashMap<String, String>>, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {}", path, e))?;

    let mut sections: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current_section = "wrap".to_string();
    sections.insert(current_section.clone(), HashMap::new());

    let mut last_key: Option<String> = None;
    for raw_line in content.lines() {
        // Preserve original to detect indentation
        let is_indented = raw_line.starts_with(' ') || raw_line.starts_with('\t');
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            // blank/comment lines also break a continuation only if blank
            if line.is_empty() {
                last_key = None;
            }
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].to_string();
            sections.entry(current_section.clone()).or_default();
            last_key = None;
        } else if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim().to_string();
            let value = line[eq_pos + 1..].trim().to_string();
            sections
                .entry(current_section.clone())
                .or_default()
                .insert(key.clone(), value);
            last_key = Some(key);
        } else if is_indented {
            if let Some(ref k) = last_key {
                let sec = sections.entry(current_section.clone()).or_default();
                if let Some(existing) = sec.get_mut(k) {
                    if !existing.is_empty() && !existing.ends_with(',') {
                        existing.push(',');
                    }
                    existing.push_str(line);
                }
            }
        }
    }

    Ok(sections)
}

/// SHA-256 of `path` as lowercase hex.
///
/// The standard library has no digest and the crate takes no hashing
/// dependency, so this runs the system `shasum -a 256` or `sha256sum` on the
/// file via stdin. When neither can run, the caller gets an error and the
/// archive is rejected: verification never silently passes.
fn sha256_file(path: &Path) -> Result<String, String> {
    const TOOLS: [(&str, &[&str]); 2] = [("shasum", &["-a", "256"]), ("sha256sum", &[])];
    let mut last_error = String::new();
    for (tool, args) in TOOLS {
        let input = std::fs::File::open(path)
            .map_err(|e| format!("Cannot read {} to verify its hash: {}", path.display(), e))?;
        match Command::new(tool)
            .args(args)
            .stdin(input)
            .stderr(std::process::Stdio::null())
            .output()
        {
            Ok(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout);
                let digest = text.split_whitespace().next().unwrap_or("");
                if digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return Ok(digest.to_ascii_lowercase());
                }
                last_error = format!("{} printed no SHA-256 digest", tool);
            }
            Ok(out) => last_error = format!("{} exited with {}", tool, out.status),
            Err(e) => last_error = format!("cannot run {}: {}", tool, e),
        }
    }
    Err(format!(
        "Cannot compute the SHA-256 of {} ({}); refusing to use an unverified archive",
        path.display(),
        last_error
    ))
}

/// Check `<what>_hash` (`source_hash` or `patch_hash`) against the bytes of
/// `path`. Like upstream Meson, the key is mandatory when the archive comes
/// from a `<what>_url` (download or cache) and optional for a local file that
/// has no URL, but a hash that is declared is always enforced.
fn verify_archive_hash(
    what: &str,
    wrap_name: &str,
    wrap: &HashMap<String, String>,
    path: &Path,
    hash_required: bool,
) -> Result<(), String> {
    let key = format!("{}_hash", what);
    let Some(expected) = wrap.get(&key) else {
        if hash_required {
            return Err(format!("Missing key '{}' in {}.wrap", key, wrap_name));
        }
        return Ok(());
    };
    let expected = expected.trim().to_ascii_lowercase();
    let actual = sha256_file(path)?;
    if actual != expected {
        return Err(format!(
            "Incorrect hash for {} in {}.wrap ({}): {} expected, {} actual",
            what, wrap_name, key, expected, actual
        ));
    }
    Ok(())
}

/// An archive that has been located and hash-checked but not yet extracted.
struct Archive {
    path: std::path::PathBuf,
    /// A copy or download made by this call, deleted again on failure.
    created: bool,
}

/// Locate the `what` archive (`source` or `patch`), fetching it when it is
/// not on disk, and verify its hash. Returns `None` when it is absent and
/// there is no URL to fetch it from. Nothing is extracted here, so a hash
/// mismatch leaves no wrap-controlled bytes in the tree.
fn fetch_archive(
    what: &str,
    wrap_name: &str,
    wrap: &HashMap<String, String>,
    parent: &Path,
    filename: &str,
    url: Option<&String>,
    network: bool,
) -> Result<Option<Archive>, String> {
    let path = parent.join(filename);
    let mut created = false;
    if !path.exists() {
        // packagefiles/ is the repository's own tree; packagecache/ is where
        // Meson caches downloads. Both are verified like a fresh download.
        let packagefiles_path = parent.join("packagefiles").join(filename);
        let packagecache_path = parent.join("packagecache").join(filename);
        let local = [packagefiles_path, packagecache_path]
            .into_iter()
            .find(|candidate| candidate.exists());
        if let Some(local) = local {
            std::fs::copy(&local, &path)
                .map_err(|e| format!("Failed to copy local archive: {}", e))?;
            created = true;
        } else if let Some(url) = url {
            if !network {
                return Err(format!(
                    "Automatic wrap-based subproject downloading is disabled (wrap_mode=nodownload): {}.wrap needs {} for {}",
                    wrap_name, url, what
                ));
            }
            eprintln!("Downloading {}...", filename);
            let status = Command::new("curl")
                .args(["-L", "-o"])
                .arg(&path)
                .arg(url)
                .status()
                .map_err(|e| format!("Failed to download: {}", e));
            created = true;
            match status {
                Ok(status) if status.success() => {}
                other => {
                    let _ = std::fs::remove_file(&path);
                    return Err(other.err().unwrap_or_else(|| "Download failed".to_string()));
                }
            }
        } else {
            return Ok(None);
        }
    }

    // Upstream Meson (wrap.py `_get_file_internal`) requires the hash for
    // anything tied to a `<what>_url` and only tolerates a missing one for a
    // local file shipped in packagefiles/.
    if let Err(message) = verify_archive_hash(what, wrap_name, wrap, &path, url.is_some()) {
        if created {
            let _ = std::fs::remove_file(&path);
        }
        return Err(message);
    }
    Ok(Some(Archive { path, created }))
}

fn download_file_wrap(
    config: &HashMap<String, HashMap<String, String>>,
    dest_dir: &str,
    wrap_name: &str,
    network: bool,
) -> Result<(), String> {
    let wrap = config.get("wrap").ok_or("No [wrap] section")?;

    let url = wrap.get("source_url");
    let filename = wrap
        .get("source_filename")
        .ok_or("No source_filename in wrap file")?;
    let directory = wrap.get("directory").map(|s| s.as_str()).unwrap_or("");

    let parent = Path::new(dest_dir).parent().unwrap_or(Path::new("."));

    let source = fetch_archive("source", wrap_name, wrap, parent, filename, url, network)?
        .ok_or_else(|| {
            format!(
                "No source_url in wrap file and local file not found: {}",
                parent.join(filename).display()
            )
        })?;
    let archive_path = source.path.clone();

    // Resolve and verify the patch archive as well, before anything is
    // extracted, so a bad patch cannot leave a half-built subproject behind.
    let patch_filename_explicit = wrap.get("patch_filename").map(|s| s.as_str());
    let patch_url = wrap.get("patch_url");
    let patch = if patch_url.is_some() || patch_filename_explicit.is_some() {
        let patch_filename = patch_filename_explicit.unwrap_or("patch.tar.gz");
        match fetch_archive(
            "patch",
            wrap_name,
            wrap,
            parent,
            patch_filename,
            patch_url,
            network,
        ) {
            Ok(patch) => patch,
            Err(message) => {
                if source.created {
                    let _ = std::fs::remove_file(&archive_path);
                }
                return Err(message);
            }
        }
    } else {
        None
    };

    // Extract
    let _extract_dir = if directory.is_empty() {
        dest_dir
    } else {
        directory
    };
    let lead_missing = wrap
        .get("lead_directory_missing")
        .map(|s| s == "true")
        .unwrap_or(false);
    // If lead_directory_missing=true, extract into the wrap-declared directory itself
    // rather than the parent (since the archive has no top-level dir).
    let extract_target: std::path::PathBuf = if lead_missing && !directory.is_empty() {
        let p = parent.join(directory);
        std::fs::create_dir_all(&p).ok();
        p
    } else {
        // Create dest_dir as well (for archives that include their own top dir
        // matching directory= or otherwise).
        let _ = std::fs::create_dir_all(dest_dir);
        parent.to_path_buf()
    };

    eprintln!("Extracting {}...", filename);
    let fname = filename.to_lowercase();
    if fname.ends_with(".tar.gz") || fname.ends_with(".tgz") {
        Command::new("tar")
            .args(["xzf"])
            .arg(&archive_path)
            .arg("-C")
            .arg(&extract_target)
            .status()
            .map_err(|e| format!("Extract failed: {}", e))?;
    } else if fname.ends_with(".tar.xz") || fname.ends_with(".txz") {
        Command::new("tar")
            .args(["xJf"])
            .arg(&archive_path)
            .arg("-C")
            .arg(&extract_target)
            .status()
            .map_err(|e| format!("Extract failed: {}", e))?;
    } else if fname.ends_with(".tar.bz2") {
        Command::new("tar")
            .args(["xjf"])
            .arg(&archive_path)
            .arg("-C")
            .arg(&extract_target)
            .status()
            .map_err(|e| format!("Extract failed: {}", e))?;
    } else if fname.ends_with(".zip") {
        Command::new("unzip")
            .arg("-o")
            .arg(&archive_path)
            .arg("-d")
            .arg(&extract_target)
            .status()
            .map_err(|e| format!("Extract failed: {}", e))?;
    }

    // Apply patch_directory: copy contents of packagefiles/<patch_directory>/ into the extracted dir
    if let Some(patch_dir) = wrap.get("patch_directory") {
        let src = parent.join("packagefiles").join(patch_dir);
        if src.is_dir() {
            fn copy_dir_recursive(
                src: &std::path::Path,
                dst: &std::path::Path,
            ) -> std::io::Result<()> {
                std::fs::create_dir_all(dst)?;
                for entry in std::fs::read_dir(src)? {
                    let e = entry?;
                    let to = dst.join(e.file_name());
                    if e.file_type()?.is_dir() {
                        copy_dir_recursive(&e.path(), &to)?;
                    } else {
                        std::fs::copy(e.path(), &to)?;
                    }
                }
                Ok(())
            }
            let _ = copy_dir_recursive(&src, &extract_target);
        }
    }

    // Apply the verified patch archive if present
    if let Some(patch) = patch {
        let patch_path = patch.path;
        let pname = patch_path.to_string_lossy().to_lowercase();
        let extract_dir_arg = if wrap.get("patch_directory").is_some()
            || patch_filename_explicit.is_some()
        {
            // patches typically extract one level above
            parent.to_path_buf()
        } else {
            std::path::PathBuf::from(dest_dir)
        };
        if pname.ends_with(".tar.xz") || pname.ends_with(".txz") {
            Command::new("tar")
                .arg("xJf")
                .arg(&patch_path)
                .arg("-C")
                .arg(&extract_dir_arg)
                .status()
                .ok();
        } else if pname.ends_with(".tar.bz2") {
            Command::new("tar")
                .arg("xjf")
                .arg(&patch_path)
                .arg("-C")
                .arg(&extract_dir_arg)
                .status()
                .ok();
        } else if pname.ends_with(".zip") {
            Command::new("unzip")
                .arg("-o")
                .arg(&patch_path)
                .arg("-d")
                .arg(&extract_dir_arg)
                .status()
                .ok();
        } else {
            Command::new("tar")
                .arg("xzf")
                .arg(&patch_path)
                .arg("-C")
                .arg(&extract_dir_arg)
                .status()
                .ok();
        }
        if patch.created {
            let _ = std::fs::remove_file(&patch_path);
        }
    }

    // Apply diff files. Each entry is resolved relative to (in order):
    //   subprojects/packagefiles/<entry>
    //   subprojects/<entry>
    if let Some(diff_files) = wrap.get("diff_files") {
        for diff in diff_files.split([',', '\n']) {
            let diff = diff.trim();
            if diff.is_empty() {
                continue;
            }
            let candidate1 = parent.join("packagefiles").join(diff);
            let candidate2 = parent.join(diff);
            let diff_path = if candidate1.is_file() {
                candidate1
            } else if candidate2.is_file() {
                candidate2
            } else {
                continue;
            };
            Command::new("patch")
                .args(["-p1", "-i"])
                .arg(&diff_path)
                .current_dir(&extract_target)
                .status()
                .map_err(|e| format!("Patch failed: {}", e))?;
        }
    }

    let _ = std::fs::remove_file(&archive_path);
    Ok(())
}

fn download_git_wrap(
    config: &HashMap<String, HashMap<String, String>>,
    dest_dir: &str,
) -> Result<(), String> {
    let wrap = config.get("wrap").ok_or("No [wrap] section")?;

    let url = wrap.get("url").ok_or("No url in git wrap")?;
    let revision = wrap.get("revision").unwrap_or(&"HEAD".to_string()).clone();
    let depth = wrap.get("depth");

    eprintln!("Cloning {}...", url);
    let mut cmd = Command::new("git");
    cmd.arg("clone").arg(url).arg(dest_dir);
    if let Some(depth) = depth {
        cmd.arg("--depth").arg(depth);
    }

    let status = cmd
        .status()
        .map_err(|e| format!("Git clone failed: {}", e))?;

    if !status.success() {
        return Err("Git clone failed".to_string());
    }

    // Checkout specific revision
    if revision != "HEAD" {
        Command::new("git")
            .args(["checkout", &revision])
            .current_dir(dest_dir)
            .status()
            .map_err(|e| format!("Git checkout failed: {}", e))?;
    }

    Ok(())
}

fn download_hg_wrap(
    config: &HashMap<String, HashMap<String, String>>,
    dest_dir: &str,
) -> Result<(), String> {
    let wrap = config.get("wrap").ok_or("No [wrap] section")?;
    let url = wrap.get("url").ok_or("No url in hg wrap")?;
    let revision = wrap.get("revision");

    let mut cmd = Command::new("hg");
    cmd.arg("clone").arg(url).arg(dest_dir);
    if let Some(rev) = revision {
        cmd.arg("-r").arg(rev);
    }

    cmd.status()
        .map_err(|e| format!("Hg clone failed: {}", e))?;
    Ok(())
}

fn download_svn_wrap(
    config: &HashMap<String, HashMap<String, String>>,
    dest_dir: &str,
) -> Result<(), String> {
    let wrap = config.get("wrap").ok_or("No [wrap] section")?;
    let url = wrap.get("url").ok_or("No url in svn wrap")?;
    let revision = wrap.get("revision");

    let mut cmd = Command::new("svn");
    cmd.arg("checkout").arg(url).arg(dest_dir);
    if let Some(rev) = revision {
        cmd.arg("-r").arg(rev);
    }

    cmd.status()
        .map_err(|e| format!("SVN checkout failed: {}", e))?;
    Ok(())
}
