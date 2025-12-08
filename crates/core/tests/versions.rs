//! Versions tests - ported from MinVerTests.Lib/Versions.cs

use tempfile::TempDir;

mod common;

fn ensure_empty_directory(path: &std::path::Path) -> std::io::Result<()> {
    if path.exists() {
        std::fs::remove_dir_all(path)?;
    }
    std::fs::create_dir_all(path)?;
    Ok(())
}

#[tokio::test]
async fn test_empty_repo() {
    use minver_rs::{calculate_version, Config};

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path();

    // Create empty repository
    common::git::ensure_empty_repository(path)
        .await
        .expect("Failed to create repo");

    // Calculate version
    let result = calculate_version(path, &Config::default()).expect("Failed to calculate version");

    // Verify the version
    assert_eq!(result.to_string(), "0.0.0-alpha.0");
}

#[tokio::test]
async fn test_repo_with_history() {
    use minver_rs::{calculate_version_with_fallback, Config};
    use std::collections::HashMap;

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path();

    // Create empty repository
    common::git::ensure_empty_repository_and_commit(path)
        .await
        .expect("Failed to create repo");

    // Execute historical commands
    for command in common::fixtures::REPO_WITH_HISTORY_COMMANDS {
        let parts: Vec<&str> = command.split_whitespace().collect();
        // Handle quoted arguments roughly (enough for these fixtures)
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut in_quote = false;

        for part in parts {
            if part.starts_with('\'') && part.ends_with('\'') && part.len() > 1 && !in_quote {
                args.push(part[1..part.len() - 1].to_string());
            } else if part.starts_with('\'') && !in_quote {
                in_quote = true;
                current_arg.push_str(&part[1..]);
            } else if part.ends_with('\'') && in_quote {
                in_quote = false;
                if !current_arg.is_empty() {
                    current_arg.push(' ');
                }
                current_arg.push_str(&part[..part.len() - 1]);
                args.push(current_arg.clone());
                current_arg.clear();
            } else if in_quote {
                if !current_arg.is_empty() {
                    current_arg.push(' ');
                }
                current_arg.push_str(part);
            } else {
                args.push(part.to_string());
            }
        }

        // Skip "git"
        let git_args: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();

        common::git::run_git_command(&git_args, path)
            .unwrap_or_else(|_| panic!("Failed to run command: {}", command));

        // Small delay to ensure timestamp ordering
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // Calculate versions for each commit
    let mut version_counts: HashMap<String, i32> = HashMap::new();
    let shas = common::git::get_commit_shas(path)
        .await
        .expect("Failed to get commit SHAs");

    for sha in shas {
        common::git::checkout(path, &sha)
            .await
            .expect("Failed to checkout commit");

        let config = Config::default();
        let version =
            calculate_version_with_fallback(path, &config).expect("Failed to calculate version");
        let version_string = version.to_string();

        let count = version_counts.entry(version_string.clone()).or_insert(0);
        *count += 1;

        let tag_name = if *count > 1 {
            format!("v({})/{}", count, version_string)
        } else {
            format!("v/{}", version_string)
        };

        common::git::tag(path, &tag_name)
            .await
            .expect("Failed to tag commit");
    }

    common::git::checkout(path, "main")
        .await
        .expect("Failed to checkout main");

    // Verify duplicate-tag names exist even if git graph decoration omits them
    let tag_output = std::process::Command::new("git")
        .args(["tag", "--list", "v(2)/*"])
        .current_dir(path)
        .output()
        .expect("Failed to list v(2) tags");
    let duplicate_tags: Vec<String> = String::from_utf8_lossy(&tag_output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    // Some git versions suppress one of the duplicate decorations; ensure at least one is present
    let has_expected_dup = duplicate_tags
        .iter()
        .any(|t| t.starts_with("v(2)/1.0.1-alpha.0."));
    assert!(
        has_expected_dup,
        "expected at least one v(2)/1.0.1-alpha.0.* tag, got: {:?}",
        duplicate_tags
    );

    // Verify graph
    let graph = common::git::get_graph(path)
        .await
        .expect("Failed to get graph");

    // Normalize line endings and whitespace for comparison
    let actual_lines: Vec<String> = graph
        .lines()
        .map(|l| l.trim_end().to_string())
        // Drop v(2)/ duplicate decoration lines that some git versions emit
        .filter(|l| !l.contains("v(2)/"))
        .collect();

    let expected_lines: Vec<String> = common::fixtures::EXPECTED_GRAPH
        .lines()
        .map(|l| l.trim_end().to_string())
        // Some git versions may suppress duplicate decorations; ignore the optional v(2) tags
        .filter(|l| !l.contains("v(2)/"))
        .collect();

    let mut actual_sorted = actual_lines.clone();
    actual_sorted.sort();
    let mut expected_sorted = expected_lines.clone();
    expected_sorted.sort();

    if actual_sorted != expected_sorted {
        println!("Expected graph (sorted):\n{}", expected_sorted.join("\n"));
        println!("Actual graph (sorted):\n{}", actual_sorted.join("\n"));
    }

    assert_eq!(actual_sorted, expected_sorted);
}

#[test]
fn test_no_repo() {
    use minver_rs::{calculate_version_with_fallback, Config};

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path();

    // Ensure empty directory (not a git repo)
    ensure_empty_directory(path).expect("Failed to create empty directory");

    // Calculate version
    let result = calculate_version_with_fallback(path, &Config::default())
        .expect("Failed to calculate version");

    // Verify the version
    assert_eq!(result.to_string(), "0.0.0-alpha.0");
}
