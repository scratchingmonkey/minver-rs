//! minver-rs CLI - Command-line tool for minimalistic versioning using Git tags

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
use std::path::PathBuf;
use std::process::exit;

use minver_rs::{calculate_version, Config, MinVerError, Verbosity, VersionPart};
use tracing::{debug, error, info, warn};
use tracing_subscriber::FmtSubscriber;

shadow_rs::shadow!(build);
const CLI_VERSION: &str = env!("MINVER_CALCULATED_VERSION");

/// minver-rs - Minimalistic versioning using Git tags
#[derive(Parser, Debug, Clone)]
#[command(name = "minver")]
#[command(about = "Calculate version numbers from Git tags")]
#[command(version = CLI_VERSION)]
struct Args {
    /// Working directory to analyze (defaults to current directory)
    #[arg(default_value = ".")]
    working_directory: PathBuf,

    /// Tag prefix to filter tags (e.g., 'v' for 'v1.0.0')
    #[arg(short = 't', long = "tag-prefix")]
    tag_prefix: Option<String>,

    /// Auto-increment policy for RTM versions (major, minor, patch)
    #[arg(short = 'a', long = "auto-increment", value_parser = parse_version_part)]
    auto_increment: Option<VersionPart>,

    /// Default pre-release identifiers (e.g., 'alpha.0')
    #[arg(short = 'p', long = "default-pre-release-identifiers")]
    default_prerelease_identifiers: Option<String>,

    /// Minimum major.minor version constraint (e.g., '1.0')
    #[arg(short = 'm', long = "minimum-major-minor")]
    minimum_major_minor: Option<String>,

    /// Ignore height in version calculation
    #[arg(short = 'i', long = "ignore-height", action = ArgAction::SetTrue)]
    ignore_height: bool,

    /// Build metadata to append to versions
    #[arg(short = 'b', long = "build-metadata")]
    build_metadata: Option<String>,

    /// Verbosity level (quiet, normal, verbose, debug, trace)
    #[arg(short = 'v', long = "verbosity", value_parser = parse_verbosity)]
    verbosity: Option<Verbosity>,
}

fn parse_version_part(s: &str) -> Result<VersionPart, String> {
    s.parse::<VersionPart>()
}

fn parse_verbosity(s: &str) -> Result<Verbosity, String> {
    s.parse::<Verbosity>()
}

fn main() {
    let long_ver: &'static str = Box::leak(long_version().into_boxed_str());

    let mut cmd = Args::command();
    cmd = cmd.version(CLI_VERSION).long_version(long_ver);
    let args = Args::from_arg_matches(&cmd.get_matches()).unwrap_or_else(|e| e.exit());

    // Set up logging based on verbosity level
    let verbosity = args.verbosity.clone().unwrap_or(Verbosity::Normal);
    let tracing_level = match verbosity {
        Verbosity::Quiet => tracing::Level::ERROR,
        Verbosity::Normal => tracing::Level::WARN,
        Verbosity::Verbose => tracing::Level::INFO,
        Verbosity::Debug => tracing::Level::DEBUG,
        Verbosity::Trace => tracing::Level::TRACE,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Build configuration from CLI arguments and environment variables
    let args_clone = args.clone();
    let config = build_config(&args_clone);

    debug!("Using configuration: {:?}", config);

    // Calculate the version
    let working_dir = args.working_directory.clone();
    let result = match calculate_version(working_dir, &config) {
        Ok(result) => {
            info!("Calculated version: {}", result);
            println!("{}", result);

            if result.height > 0 && !config.ignore_height {
                debug!("Height: {}", result.height);
            }

            if !result.is_from_tag {
                debug!("Version derived from ancestor tag (not exact match)");
            }

            0 // Success exit code
        }
        Err(e) => {
            match e {
                MinVerError::GitRepoNotFound(path) => {
                    error!("'{}' is not a valid Git working directory", path);
                }
                MinVerError::NoCommits => {
                    info!("No commits found. Using default version.");
                    println!("0.0.0-alpha.0");
                }
                MinVerError::ShallowRepo => {
                    warn!("Shallow repository detected. Version calculation may be incorrect. Fetch full history with 'git fetch --unshallow'.");
                    // Still try to calculate and return the result
                    if let Ok(result) = calculate_version(args.working_directory, &config) {
                        println!("{}", result);
                    }
                }
                _ => {
                    error!("Version calculation failed: {}", e);
                }
            }
            2 // Error exit code
        }
    };

    exit(result);
}

fn build_config(args: &Args) -> Config {
    let mut config = Config::default();

    // Environment variables can override defaults
    apply_env_vars(&mut config);

    // CLI arguments take precedence over environment variables
    config.work_dir = args.working_directory.clone();

    if let Some(prefix) = &args.tag_prefix {
        config.tag_prefix = prefix.clone();
    }

    if let Some(auto_inc) = &args.auto_increment {
        config.auto_increment = auto_inc.clone();
    }

    if let Some(identifiers) = &args.default_prerelease_identifiers {
        config.default_prerelease_identifiers =
            identifiers.split('.').map(|s| s.to_string()).collect();
    }

    if let Some(min_mm) = &args.minimum_major_minor {
        if let Ok(minor_major) = minver_rs::config::MajorMinor::parse(min_mm) {
            config.minimum_major_minor = Some(minor_major);
        }
    }

    if args.ignore_height {
        config.ignore_height = true;
    }

    if let Some(build_meta) = &args.build_metadata {
        config.build_metadata = Some(build_meta.clone());
    }

    if let Some(verbosity) = &args.verbosity {
        config.verbosity = verbosity.clone().into();
    }

    config
}

fn apply_env_vars(config: &mut Config) {
    use std::env;

    if let Ok(tag_prefix) = env::var("MINVERTAGPREFIX") {
        if !tag_prefix.is_empty() {
            config.tag_prefix = tag_prefix;
        }
    }

    if let Ok(auto_inc) = env::var("MINVERAUTOINCREMENT") {
        if let Ok(part) = auto_inc.parse::<VersionPart>() {
            config.auto_increment = part;
        }
    }

    if let Ok(identifiers) = env::var("MINVERDEFAULTPRERELEASEIDENTIFIERS") {
        if !identifiers.is_empty() {
            config.default_prerelease_identifiers =
                identifiers.split('.').map(|s| s.to_string()).collect();
        }
    }

    if let Ok(min_mm) = env::var("MINVERMINIMUMMAJORMINOR") {
        if let Ok(minor_major) = minver_rs::config::MajorMinor::parse(&min_mm) {
            config.minimum_major_minor = Some(minor_major);
        }
    }

    if let Ok(ignore_height) = env::var("MINVERIGNOREHEIGHT") {
        if let Ok(value) = ignore_height.parse::<bool>() {
            config.ignore_height = value;
        }
    }

    if let Ok(build_meta) = env::var("MINVERBUILDMETADATA") {
        if !build_meta.is_empty() {
            config.build_metadata = Some(build_meta);
        }
    }

    if let Ok(verbosity) = env::var("MINVERVERBOSITY") {
        if let Ok(level) = verbosity.parse::<Verbosity>() {
            config.verbosity = level.into();
        }
    }
}

fn long_version() -> String {
    format!(
        "{version}\ncommit: {commit} ({date})\nbuild: {build}\nrustc: {rustc}",
        version = CLI_VERSION,
        commit = build::SHORT_COMMIT,
        date = build::COMMIT_DATE,
        build = build::BUILD_TIME,
        rustc = build::RUST_VERSION,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_args() {
        let args = Args {
            working_directory: PathBuf::from("/tmp"),
            tag_prefix: Some("v".to_string()),
            auto_increment: Some(VersionPart::Minor),
            default_prerelease_identifiers: Some("beta.0".to_string()),
            minimum_major_minor: Some("2.1".to_string()),
            ignore_height: true,
            build_metadata: Some("build.123".to_string()),
            verbosity: Some(Verbosity::Debug),
        };

        let config = build_config(&args);

        assert_eq!(config.work_dir, PathBuf::from("/tmp"));
        assert_eq!(config.tag_prefix, "v");
        assert_eq!(config.auto_increment, VersionPart::Minor);
        assert_eq!(config.default_prerelease_identifiers, vec!["beta", "0"]);
        assert!(config.ignore_height);
        assert_eq!(config.build_metadata, Some("build.123".to_string()));
        assert_eq!(config.verbosity, minver_rs::config::Verbosity::Debug);
    }

    #[test]
    fn test_env_var_sets_verbosity() {
        // Preserve previous value to avoid leaking state
        let original = std::env::var("MINVERVERBOSITY").ok();
        std::env::set_var("MINVERVERBOSITY", "debug");

        let args = Args {
            working_directory: PathBuf::from("."),
            tag_prefix: None,
            auto_increment: None,
            default_prerelease_identifiers: None,
            minimum_major_minor: None,
            ignore_height: false,
            build_metadata: None,
            verbosity: None,
        };

        let config = build_config(&args);
        assert_eq!(config.verbosity, minver_rs::config::Verbosity::Debug);

        if let Some(val) = original {
            std::env::set_var("MINVERVERBOSITY", val);
        } else {
            std::env::remove_var("MINVERVERBOSITY");
        }
    }
}
