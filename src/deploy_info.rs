use chrono::{DateTime, Utc};
use serde::Serialize;
/// Where the deploy script writes deploy metadata.
const DEPLOY_INFO_FILE: &str = "deploy_info";

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct DeployInfo {
    /// Unix timestamp of the deploy
    pub at: i64,
    /// Short git SHA of the deployed commit, if known
    pub sha: Option<String>,
    /// Exact UTC timestamp for display
    pub at_exact: String,
    /// Humanized relative time for display ("2 hours ago")
    pub at_human: String,
}

/// Reads the deploy info written by deploy.sh.
///
/// Falls back to the mtime of the running executable (build time) when the
/// file is missing or malformed, e.g. during local development.
/// Returns None when no information at all is available.
pub fn read() -> Option<DeployInfo> {
    match read_deploy_file(DEPLOY_INFO_FILE) {
        Some(info) => Some(info),
        None => exe_mtime_deploy_info(),
    }
}

/// deploy_info format: first line = unix timestamp, second line = short SHA
fn read_deploy_file(path: &str) -> Option<DeployInfo> {
    let contents = std::fs::read_to_string(path).ok()?;
    let mut lines = contents.lines();

    let at = lines.next()?.trim().parse::<i64>().ok()?;
    if at <= 0 {
        return None;
    }
    let sha = lines
        .next()
        .map(|sha| sha.trim().to_string())
        .filter(|sha| !sha.is_empty() && sha.len() <= 40);

    Some(deploy_info_from_timestamp(at, sha))
}

fn exe_mtime_deploy_info() -> Option<DeployInfo> {
    let exe = std::env::current_exe().ok()?;
    let mtime = std::fs::metadata(exe).ok()?.modified().ok()?;
    let at = DateTime::<Utc>::from(mtime).timestamp();

    Some(deploy_info_from_timestamp(at, None))
}

fn deploy_info_from_timestamp(at: i64, sha: Option<String>) -> DeployInfo {
    let date_time = match DateTime::from_timestamp(at, 0) {
        Some(date_time) => date_time,
        None => return deploy_info_from_timestamp(0, sha),
    };

    let human_time =
        chrono_humanize::HumanTime::from(date_time).to_text_en(
            chrono_humanize::Accuracy::Rough,
            chrono_humanize::Tense::Past,
        );

    DeployInfo {
        at,
        sha,
        at_exact: date_time.format("%Y-%m-%d %H:%M UTC").to_string(),
        at_human: human_time,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_temp_file(contents: &str) -> String {
        let path = std::env::temp_dir().join(format!(
            "deploy_info_test_{}",
            std::process::id()
        ));
        std::fs::write(&path, contents).unwrap();
        path.to_string_lossy().to_string()
    }

    #[test]
    fn test_read_deploy_file_valid() {
        let path = write_temp_file("1700000000\nabc1234\n");
        let info = read_deploy_file(&path).unwrap();
        assert_eq!(info.at, 1700000000);
        assert_eq!(info.sha.as_deref(), Some("abc1234"));
        assert!(!info.at_human.is_empty());
        assert!(info.at_exact.contains("UTC"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_read_deploy_file_missing_sha() {
        let path = write_temp_file("1700000000\n");
        let info = read_deploy_file(&path).unwrap();
        assert_eq!(info.sha, None);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_read_deploy_file_garbage_timestamp() {
        let path = write_temp_file("not-a-number\nabc1234\n");
        assert!(read_deploy_file(&path).is_none());
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_read_deploy_file_zero_timestamp() {
        let path = write_temp_file("0\nabc1234\n");
        assert!(read_deploy_file(&path).is_none());
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_read_deploy_file_sha_too_long() {
        let path = write_temp_file("1700000000\n0123456789012345678901234567890123456789012345\n");
        let info = read_deploy_file(&path).unwrap();
        assert_eq!(info.sha, None);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_read_deploy_file_missing_file() {
        assert!(read_deploy_file("/nonexistent/deploy_info").is_none());
    }

    #[test]
    fn test_fallback_has_timestamp() {
        let info = exe_mtime_deploy_info().unwrap();
        let now = Utc::now().timestamp();
        // The binary was built at some point in the past, not in the future
        assert!(info.at <= now);
        assert_eq!(info.sha, None);
    }
}
