use std::{env, fs, path::Path, time::Duration};

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::output;

/// Release API of the repository. GitHub serves the latest published release,
/// drafts and pre-releases excluded.
const RELEASE_URL: &str = "https://api.github.com/repos/aaimeraud/aarshi/releases/latest";

/// Sent to GitHub so the traffic identifies aarshi. Also required by the API.
const USER_AGENT: &str = concat!("aarshi/", env!("CARGO_PKG_VERSION"));

/// Upper bound for the metadata response, and for the binary itself.
const METADATA_LIMIT: u64 = 1024 * 1024;
const BINARY_LIMIT: u64 = 100 * 1024 * 1024;

pub struct UpdateOptions {
    /// Report the latest version without installing it.
    pub check: bool,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    /// `sha256:<hex>` when GitHub computed a digest for the asset.
    digest: Option<String>,
}

pub fn run(options: UpdateOptions) -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");
    let release = latest_release()?;
    let latest = release.tag_name.trim_start_matches('v');

    if !is_newer(latest, current) {
        println!(
            "{}",
            output::success(&format!("aarshi {current} is already the latest version"))
        );
        return Ok(());
    }

    if options.check {
        println!(
            "{}",
            output::info(&format!(
                "aarshi {latest} is available (current {current}): {}",
                release.html_url
            ))
        );
        return Ok(());
    }

    println!(
        "{}",
        output::info(&format!(
            "updating aarshi {current} to {latest} from {}",
            release.html_url
        ))
    );

    let name = asset_name()?;
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == name)
        .with_context(|| format!("release {latest} has no asset named {name}"))?;
    let bytes = download(&asset.browser_download_url)?;
    verify(&bytes, asset.digest.as_deref())?;
    install(&bytes)?;

    println!(
        "{}",
        output::success(&format!(
            "aarshi updated to {latest}; run `aarshi --version` to check"
        ))
    );
    Ok(())
}

/// Reads the latest release from the GitHub API.
fn latest_release() -> Result<Release> {
    let mut response = agent()
        .get(RELEASE_URL)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .call()
        .context("cannot reach the GitHub API; check your connection")?;
    let status = response.status();
    let body = response
        .body_mut()
        .with_config()
        .limit(METADATA_LIMIT)
        .read_to_vec()
        .context("cannot read the GitHub API response")?;

    if status == 404 {
        bail!("no release is published on GitHub yet");
    }
    if !status.is_success() {
        let reason = status.canonical_reason().unwrap_or("unexpected answer");
        let detail = serde_json::from_slice::<serde_json::Value>(&body)
            .ok()
            .and_then(|json| {
                json.get("message")
                    .and_then(|message| message.as_str())
                    .map(str::to_string)
            });
        match detail {
            Some(detail) => bail!("GitHub answered {status} ({reason}): {detail}"),
            None => bail!("GitHub answered {status} ({reason})"),
        }
    }

    serde_json::from_slice(&body).context("cannot read the latest release published on GitHub")
}

/// Downloads a release asset, which is a raw binary, not an archive.
fn download(url: &str) -> Result<Vec<u8>> {
    let mut response = agent()
        .get(url)
        .header("User-Agent", USER_AGENT)
        .call()
        .with_context(|| format!("cannot download {url}; check your connection"))?;
    let status = response.status();
    let bytes = response
        .body_mut()
        .with_config()
        .limit(BINARY_LIMIT)
        .read_to_vec()
        .with_context(|| format!("cannot read the download of {url}"))?;
    if !status.is_success() {
        bail!("the download of {url} failed with {status}");
    }
    Ok(bytes)
}

/// A client with a bounded timeout. HTTP error statuses are returned as
/// responses so their body can explain a rate limit.
fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(120)))
        .http_status_as_error(false)
        .build()
        .into()
}

/// Checks the asset against the digest GitHub publishes when it has one.
fn verify(bytes: &[u8], digest: Option<&str>) -> Result<()> {
    let Some(expected) = digest.and_then(|digest| digest.strip_prefix("sha256:")) else {
        return Ok(());
    };
    let actual = format!("{:x}", Sha256::digest(bytes));
    if !actual.eq_ignore_ascii_case(expected) {
        bail!("the downloaded binary does not match the published sha256 checksum");
    }
    Ok(())
}

/// Writes the new binary next to the running one, then swaps it in.
fn install(bytes: &[u8]) -> Result<()> {
    let current = env::current_exe().context("cannot locate the running aarshi binary")?;
    let target = fs::canonicalize(&current)
        .with_context(|| format!("cannot resolve {}", current.display()))?;
    let dir = target
        .parent()
        .with_context(|| format!("{} has no parent folder", target.display()))?;
    let staging = dir.join(format!(".aarshi-update-{}", std::process::id()));

    fs::write(&staging, bytes).with_context(|| {
        format!(
            "cannot write {}; is the folder writable?",
            staging.display()
        )
    })?;
    make_executable(&staging)?;

    replace(&staging, &target).inspect_err(|_| {
        let _ = fs::remove_file(&staging);
    })?;
    println!(
        "{}",
        output::info(&format!("replaced {}", target.display()))
    );
    Ok(())
}

/// Replaces the running binary. On Unix the file can be renamed over while it
/// runs; Windows refuses to delete a running program but allows renaming it.
fn replace(staging: &Path, target: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        let backup = backup_path(target);
        fs::rename(target, &backup)
            .with_context(|| format!("cannot move {} out of the way", target.display()))?;
        if let Err(error) = fs::rename(staging, target) {
            let _ = fs::rename(&backup, target);
            return Err(error).with_context(|| format!("cannot write {}", target.display()));
        }
        // The old binary may still be locked until this process exits.
        let _ = fs::remove_file(&backup);
    }
    #[cfg(not(windows))]
    {
        fs::rename(staging, target).with_context(|| {
            format!(
                "cannot replace {}; check the folder permissions",
                target.display()
            )
        })?;
    }
    Ok(())
}

#[cfg(windows)]
fn backup_path(target: &Path) -> std::path::PathBuf {
    let name = target
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "aarshi.exe".to_string());
    target.with_file_name(format!("{name}.old-{}", std::process::id()))
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .with_context(|| format!("cannot make {} executable", path.display()))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

/// The release asset matching the machine aarshi runs on.
fn asset_name() -> Result<String> {
    asset_name_for(env::consts::OS, env::consts::ARCH)
}

fn asset_name_for(os: &str, arch: &str) -> Result<String> {
    let triple = match (os, arch) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        (os, arch) => bail!(
            "no prebuilt binary is published for {os}/{arch}; build from source with `cargo install --path .`"
        ),
    };
    Ok(format!("aarshi-{triple}{}", env::consts::EXE_SUFFIX))
}

fn is_newer(candidate: &str, current: &str) -> bool {
    match (parse_version(candidate), parse_version(current)) {
        (Some(candidate), Some(current)) => candidate > current,
        _ => candidate != current,
    }
}

fn parse_version(text: &str) -> Option<semver::Version> {
    semver::Version::parse(text.trim_start_matches('v')).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_versions_are_detected() {
        assert!(is_newer("1.1.0", "1.0.0"));
        assert!(is_newer("1.0.1", "1.0.0"));
        assert!(is_newer("v2.0.0", "1.9.9"));
        assert!(is_newer("1.0.0", "1.0.0-beta.1"));
    }

    #[test]
    fn same_or_older_versions_are_ignored() {
        assert!(!is_newer("1.0.0", "1.0.0"));
        assert!(!is_newer("0.9.0", "1.0.0"));
        assert!(!is_newer("v1.0.0", "1.0.0"));
    }

    #[test]
    fn assets_follow_the_published_names() {
        assert_eq!(
            asset_name_for("macos", "aarch64").unwrap(),
            format!("aarshi-aarch64-apple-darwin{}", env::consts::EXE_SUFFIX)
        );
        assert_eq!(
            asset_name_for("linux", "x86_64").unwrap(),
            format!("aarshi-x86_64-unknown-linux-gnu{}", env::consts::EXE_SUFFIX)
        );
        assert!(asset_name_for("linux", "aarch64").is_err());
    }

    #[test]
    fn a_matching_digest_passes() {
        let bytes = b"aarshi";
        let digest = format!("sha256:{:x}", Sha256::digest(bytes));
        assert!(verify(bytes, Some(&digest)).is_ok());
        assert!(verify(bytes, None).is_ok());
    }

    #[test]
    fn a_different_digest_is_refused() {
        assert!(verify(b"aarshi", Some(&format!("sha256:{}", "0".repeat(64)))).is_err());
    }
}
