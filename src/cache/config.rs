use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use bytesize::ByteSize;
use serde::Deserialize;

use crate::error::BioMcpError;

const DEFAULT_MAX_SIZE: u64 = 10_000_000_000;
const DEFAULT_MAX_AGE_SECS: u64 = 86_400;
const DEFAULT_MIN_DISK_FREE: DiskFreeThreshold = DiskFreeThreshold::Percent(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiskFreeThreshold {
    Percent(u8),
    Bytes(u64),
}

impl DiskFreeThreshold {
    pub(crate) fn required_free_bytes(&self, total_bytes: u64) -> u64 {
        match self {
            Self::Percent(percent) => total_bytes.saturating_mul(u64::from(*percent)) / 100,
            Self::Bytes(bytes) => *bytes,
        }
    }

    pub(crate) fn is_violated(&self, available_bytes: u64, total_bytes: u64) -> bool {
        available_bytes < self.required_free_bytes(total_bytes)
    }

    pub(crate) fn display(&self) -> String {
        match self {
            Self::Percent(percent) => format!("{percent}%"),
            Self::Bytes(bytes) => ByteSize(*bytes).to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfigOrigin {
    Env,
    File,
    Default,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CacheConfigOrigins {
    pub(crate) cache_root: ConfigOrigin,
    pub(crate) max_size: ConfigOrigin,
    pub(crate) min_disk_free: ConfigOrigin,
    pub(crate) max_age: ConfigOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedCacheConfig {
    pub(crate) cache_root: PathBuf,
    pub(crate) max_size: u64,
    pub(crate) min_disk_free: DiskFreeThreshold,
    pub(crate) max_age: Duration,
    pub(crate) origins: CacheConfigOrigins,
}

pub(crate) type CacheConfig = ResolvedCacheConfig;

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct CacheToml {
    #[serde(default)]
    cache: CacheTomlSection,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct CacheTomlSection {
    dir: Option<String>,
    max_size: Option<u64>,
    min_disk_free: Option<String>,
    max_age_secs: Option<u64>,
}

pub(crate) fn resolve_cache_config() -> Result<CacheConfig, BioMcpError> {
    let env_dir = std::env::var("BIOMCP_CACHE_DIR").ok();
    let env_max_size = std::env::var("BIOMCP_CACHE_MAX_SIZE").ok();
    let env_min_disk_free = std::env::var("BIOMCP_CACHE_MIN_DISK_FREE").ok();
    let env_max_age = std::env::var("BIOMCP_CACHE_MAX_AGE").ok();
    let default_cache_root = default_cache_root();
    let config_path = config_file_path();
    let toml_content = match config_path.as_deref() {
        Some(path) => read_cache_toml(path)?,
        None => None,
    };

    resolve_cache_config_with_source(
        env_dir.as_deref(),
        env_max_size.as_deref(),
        env_min_disk_free.as_deref(),
        env_max_age.as_deref(),
        toml_content.as_deref(),
        default_cache_root,
        config_path.as_deref(),
    )
}

fn resolve_cache_config_from_parts(
    env_dir: Option<&str>,
    env_max_size: Option<&str>,
    toml_content: Option<&str>,
    default_cache_root: PathBuf,
) -> Result<CacheConfig, BioMcpError> {
    resolve_cache_config_from_parts_with_max_age(
        env_dir,
        env_max_size,
        None,
        toml_content,
        default_cache_root,
    )
}

fn resolve_cache_config_from_parts_with_max_age(
    env_dir: Option<&str>,
    env_max_size: Option<&str>,
    env_max_age: Option<&str>,
    toml_content: Option<&str>,
    default_cache_root: PathBuf,
) -> Result<CacheConfig, BioMcpError> {
    resolve_cache_config_with_source(
        env_dir,
        env_max_size,
        None,
        env_max_age,
        toml_content,
        default_cache_root,
        None,
    )
}

fn default_cache_root() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("biomcp")
}

fn config_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("biomcp").join("cache.toml"))
}

fn resolve_cache_config_with_source(
    env_dir: Option<&str>,
    env_max_size: Option<&str>,
    env_min_disk_free: Option<&str>,
    env_max_age: Option<&str>,
    toml_content: Option<&str>,
    default_cache_root: PathBuf,
    config_path: Option<&std::path::Path>,
) -> Result<CacheConfig, BioMcpError> {
    let CacheToml {
        cache:
            CacheTomlSection {
                dir: toml_dir,
                max_size: toml_max_size,
                min_disk_free: toml_min_disk_free,
                max_age_secs: toml_max_age_secs,
            },
    } = parse_cache_toml(toml_content, config_path)?;

    let (cache_root, cache_root_origin) = if let Some(dir) = normalize_env_value(env_dir) {
        (PathBuf::from(dir), ConfigOrigin::Env)
    } else if let Some(dir) = parse_toml_dir(toml_dir.as_deref(), config_path)? {
        (dir, ConfigOrigin::File)
    } else {
        (default_cache_root, ConfigOrigin::Default)
    };

    let (max_size, max_size_origin) = if let Some(size) = parse_env_max_size(env_max_size)? {
        (size, ConfigOrigin::Env)
    } else if let Some(size) =
        parse_toml_positive_u64(toml_max_size, "[cache].max_size", config_path)?
    {
        (size, ConfigOrigin::File)
    } else {
        (DEFAULT_MAX_SIZE, ConfigOrigin::Default)
    };

    let (min_disk_free, min_disk_free_origin) =
        if let Some(threshold) = parse_env_min_disk_free(env_min_disk_free)? {
            (threshold, ConfigOrigin::Env)
        } else if let Some(threshold) =
            parse_toml_min_disk_free(toml_min_disk_free.as_deref(), config_path)?
        {
            (threshold, ConfigOrigin::File)
        } else {
            (DEFAULT_MIN_DISK_FREE, ConfigOrigin::Default)
        };

    let (max_age_secs, max_age_origin) = if let Some(age_secs) = parse_env_max_age(env_max_age)? {
        (age_secs, ConfigOrigin::Env)
    } else if let Some(age_secs) =
        parse_toml_positive_u64(toml_max_age_secs, "[cache].max_age_secs", config_path)?
    {
        (age_secs, ConfigOrigin::File)
    } else {
        (DEFAULT_MAX_AGE_SECS, ConfigOrigin::Default)
    };

    Ok(ResolvedCacheConfig {
        cache_root,
        max_size,
        min_disk_free,
        max_age: Duration::from_secs(max_age_secs),
        origins: CacheConfigOrigins {
            cache_root: cache_root_origin,
            max_size: max_size_origin,
            min_disk_free: min_disk_free_origin,
            max_age: max_age_origin,
        },
    })
}

fn read_cache_toml(path: &std::path::Path) -> Result<Option<String>, BioMcpError> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(BioMcpError::Io(std::io::Error::new(
            err.kind(),
            format!("failed to read {}: {}", path.display(), err),
        ))),
    }
}

fn parse_cache_toml(
    toml_content: Option<&str>,
    config_path: Option<&std::path::Path>,
) -> Result<CacheToml, BioMcpError> {
    let Some(toml_content) = toml_content else {
        return Ok(CacheToml::default());
    };

    if toml_content.trim().is_empty() {
        return Ok(CacheToml::default());
    }

    toml::from_str::<CacheToml>(toml_content)
        .map_err(|err| invalid_config(config_path, format!("failed to parse cache config: {err}")))
}

fn normalize_env_value(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn parse_env_max_size(value: Option<&str>) -> Result<Option<u64>, BioMcpError> {
    let Some(value) = normalize_env_value(value) else {
        return Ok(None);
    };

    let parsed = value.parse::<u64>().map_err(|_| {
        BioMcpError::InvalidArgument(format!(
            "BIOMCP_CACHE_MAX_SIZE must be a positive integer number of bytes: got '{value}'"
        ))
    })?;

    if parsed == 0 {
        return Err(BioMcpError::InvalidArgument(
            "BIOMCP_CACHE_MAX_SIZE must be greater than 0".into(),
        ));
    }

    Ok(Some(parsed))
}

fn parse_env_min_disk_free(value: Option<&str>) -> Result<Option<DiskFreeThreshold>, BioMcpError> {
    let Some(value) = normalize_env_value(value) else {
        return Ok(None);
    };

    parse_disk_free_threshold(value)
        .map(Some)
        .map_err(|message| {
            BioMcpError::InvalidArgument(format!(
                "BIOMCP_CACHE_MIN_DISK_FREE {message}: got '{value}'"
            ))
        })
}

fn parse_env_max_age(value: Option<&str>) -> Result<Option<u64>, BioMcpError> {
    let Some(value) = normalize_env_value(value) else {
        return Ok(None);
    };

    let parsed = value.parse::<u64>().map_err(|_| {
        BioMcpError::InvalidArgument(format!(
            "BIOMCP_CACHE_MAX_AGE must be a positive integer number of seconds: got '{value}'"
        ))
    })?;

    if parsed == 0 {
        return Err(BioMcpError::InvalidArgument(
            "BIOMCP_CACHE_MAX_AGE must be greater than 0".into(),
        ));
    }

    Ok(Some(parsed))
}

fn parse_toml_dir(
    value: Option<&str>,
    config_path: Option<&std::path::Path>,
) -> Result<Option<PathBuf>, BioMcpError> {
    let Some(value) = value else {
        return Ok(None);
    };

    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(invalid_config(
            config_path,
            "[cache].dir must not be blank".into(),
        ));
    }

    Ok(Some(PathBuf::from(trimmed)))
}

fn parse_toml_positive_u64(
    value: Option<u64>,
    field_name: &str,
    config_path: Option<&std::path::Path>,
) -> Result<Option<u64>, BioMcpError> {
    match value {
        Some(0) => Err(invalid_config(
            config_path,
            format!("{field_name} must be greater than 0"),
        )),
        Some(value) => Ok(Some(value)),
        None => Ok(None),
    }
}

fn parse_toml_min_disk_free(
    value: Option<&str>,
    config_path: Option<&std::path::Path>,
) -> Result<Option<DiskFreeThreshold>, BioMcpError> {
    let Some(value) = value else {
        return Ok(None);
    };

    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(invalid_config(
            config_path,
            "[cache].min_disk_free must not be blank".into(),
        ));
    }

    parse_disk_free_threshold(trimmed)
        .map(Some)
        .map_err(|message| invalid_config(config_path, format!("[cache].min_disk_free {message}")))
}

fn parse_disk_free_threshold(value: &str) -> Result<DiskFreeThreshold, &'static str> {
    if let Some(percent) = value.strip_suffix('%') {
        let parsed = percent.trim().parse::<u8>().map_err(
            |_| "must be a whole-number percent like 10% or a positive byte size like 5G",
        )?;
        if parsed == 0 {
            return Err("must be greater than 0");
        }
        if parsed > 100 {
            return Err("percent must be between 1 and 100");
        }
        return Ok(DiskFreeThreshold::Percent(parsed));
    }

    let parsed = ByteSize::from_str(value)
        .map_err(|_| "must be a whole-number percent like 10% or a positive byte size like 5G")?
        .as_u64();
    if parsed == 0 {
        return Err("must be greater than 0");
    }
    Ok(DiskFreeThreshold::Bytes(parsed))
}

fn invalid_config(config_path: Option<&std::path::Path>, message: String) -> BioMcpError {
    BioMcpError::InvalidArgument(format!("{}: {message}", config_source_label(config_path)))
}

fn config_source_label(config_path: Option<&std::path::Path>) -> String {
    config_path
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "cache.toml".to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        CacheConfig, CacheConfigOrigins, ConfigOrigin, DEFAULT_MAX_AGE_SECS, DEFAULT_MAX_SIZE,
        DEFAULT_MIN_DISK_FREE, DiskFreeThreshold, read_cache_toml, resolve_cache_config_from_parts,
        resolve_cache_config_from_parts_with_max_age, resolve_cache_config_with_source,
    };
    use crate::error::BioMcpError;
    use crate::test_support::TempDirGuard;
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::time::Duration;

    fn default_config_with_root(root: impl Into<PathBuf>) -> CacheConfig {
        CacheConfig {
            cache_root: root.into(),
            max_size: DEFAULT_MAX_SIZE,
            min_disk_free: DEFAULT_MIN_DISK_FREE,
            max_age: Duration::from_secs(DEFAULT_MAX_AGE_SECS),
            origins: CacheConfigOrigins {
                cache_root: ConfigOrigin::Default,
                max_size: ConfigOrigin::Default,
                min_disk_free: ConfigOrigin::Default,
                max_age: ConfigOrigin::Default,
            },
        }
    }

    fn assert_invalid_argument_contains(err: BioMcpError, expected: &[&str]) {
        let message = err.to_string();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        for needle in expected {
            assert!(
                message.contains(needle),
                "expected error message to contain {needle:?}, got: {message}"
            );
        }
    }

    #[test]
    fn defaults_when_no_env_or_file_uses_default_cache_config() {
        let default_root = PathBuf::from("/tmp/default-cache");
        let config =
            resolve_cache_config_from_parts(None, None, None, default_root.clone()).expect("ok");
        assert_eq!(config, default_config_with_root(default_root));
    }

    #[test]
    fn env_cache_dir_overrides_file_and_default() {
        let default_root = PathBuf::from("/tmp/default-cache");
        let config = resolve_cache_config_from_parts(
            Some("  /env-cache  "),
            None,
            Some("[cache]\ndir = \"/file-cache\"\n"),
            default_root,
        )
        .expect("ok");
        assert_eq!(config.cache_root, PathBuf::from("/env-cache"));
        assert_eq!(config.origins.cache_root, ConfigOrigin::Env);
    }

    #[test]
    fn env_max_size_overrides_file_and_default() {
        let config = resolve_cache_config_from_parts(
            None,
            Some(" 5000 "),
            Some("[cache]\nmax_size = 42\n"),
            PathBuf::from("/tmp/default-cache"),
        )
        .expect("ok");
        assert_eq!(config.max_size, 5_000);
        assert_eq!(config.origins.max_size, ConfigOrigin::Env);
    }

    #[test]
    fn toml_dir_overrides_default() {
        let config = resolve_cache_config_from_parts(
            None,
            None,
            Some("[cache]\ndir = \"relative-cache\"\n"),
            PathBuf::from("/tmp/default-cache"),
        )
        .expect("ok");
        assert_eq!(config.cache_root, PathBuf::from("relative-cache"));
        assert_eq!(config.origins.cache_root, ConfigOrigin::File);
    }

    #[test]
    fn toml_max_size_overrides_default() {
        let config = resolve_cache_config_from_parts(
            None,
            None,
            Some("[cache]\nmax_size = 1234\n"),
            PathBuf::from("/tmp/default-cache"),
        )
        .expect("ok");
        assert_eq!(config.max_size, 1_234);
        assert_eq!(config.origins.max_size, ConfigOrigin::File);
    }

    #[test]
    fn toml_max_age_overrides_default() {
        let config = resolve_cache_config_from_parts(
            None,
            None,
            Some("[cache]\nmax_age_secs = 172800\n"),
            PathBuf::from("/tmp/default-cache"),
        )
        .expect("ok");
        assert_eq!(config.max_age, Duration::from_secs(172_800));
        assert_eq!(config.origins.max_age, ConfigOrigin::File);
    }

    #[test]
    fn env_max_age_overrides_file_and_default() {
        let config = resolve_cache_config_from_parts_with_max_age(
            None,
            None,
            Some(" 172800 "),
            Some("[cache]\nmax_age_secs = 7200\n"),
            PathBuf::from("/tmp/default-cache"),
        )
        .expect("ok");
        assert_eq!(config.max_age, Duration::from_secs(172_800));
        assert_eq!(config.origins.max_age, ConfigOrigin::Env);
    }

    #[test]
    fn blank_env_max_age_falls_through() {
        let config = resolve_cache_config_from_parts_with_max_age(
            None,
            None,
            Some("   "),
            Some("[cache]\nmax_age_secs = 7200\n"),
            PathBuf::from("/tmp/default-cache"),
        )
        .expect("ok");
        assert_eq!(config.max_age, Duration::from_secs(7_200));
        assert_eq!(config.origins.max_age, ConfigOrigin::File);
    }

    #[test]
    fn invalid_env_max_age_returns_error() {
        let err = resolve_cache_config_from_parts_with_max_age(
            None,
            None,
            Some("foo"),
            None,
            PathBuf::from("/tmp/default"),
        )
        .expect_err("invalid env max age should fail");
        assert_invalid_argument_contains(err, &["BIOMCP_CACHE_MAX_AGE", "foo"]);
    }

    #[test]
    fn zero_env_max_age_returns_error() {
        let err = resolve_cache_config_from_parts_with_max_age(
            None,
            None,
            Some("0"),
            None,
            PathBuf::from("/tmp/default"),
        )
        .expect_err("zero env max age should fail");
        assert_invalid_argument_contains(err, &["BIOMCP_CACHE_MAX_AGE", "greater than 0"]);
    }

    #[test]
    fn origins_track_mixed_precedence_without_max_age_env_override() {
        let config = resolve_cache_config_from_parts(
            Some(" /env-cache "),
            None,
            Some("[cache]\nmax_size = 1234\nmax_age_secs = 7200\n"),
            PathBuf::from("/tmp/default-cache"),
        )
        .expect("ok");
        assert_eq!(
            config.origins,
            CacheConfigOrigins {
                cache_root: ConfigOrigin::Env,
                max_size: ConfigOrigin::File,
                min_disk_free: ConfigOrigin::Default,
                max_age: ConfigOrigin::File,
            }
        );
    }

    #[test]
    fn default_origins_are_reported_when_values_fall_through() {
        let config = resolve_cache_config_from_parts(
            Some("   "),
            Some("   "),
            Some("# empty cache section\n"),
            PathBuf::from("/tmp/default-cache"),
        )
        .expect("ok");
        assert_eq!(
            config.origins,
            CacheConfigOrigins {
                cache_root: ConfigOrigin::Default,
                max_size: ConfigOrigin::Default,
                min_disk_free: ConfigOrigin::Default,
                max_age: ConfigOrigin::Default,
            }
        );
    }

    #[test]
    fn invalid_env_size_returns_error() {
        let err =
            resolve_cache_config_from_parts(None, Some("foo"), None, PathBuf::from("/tmp/default"))
                .expect_err("invalid env size should fail");
        assert_invalid_argument_contains(err, &["BIOMCP_CACHE_MAX_SIZE", "foo"]);
    }

    #[test]
    fn zero_env_size_returns_error() {
        let err =
            resolve_cache_config_from_parts(None, Some("0"), None, PathBuf::from("/tmp/default"))
                .expect_err("zero env size should fail");
        assert_invalid_argument_contains(err, &["BIOMCP_CACHE_MAX_SIZE", "greater than 0"]);
    }

    #[test]
    fn invalid_toml_syntax_returns_error() {
        let err = resolve_cache_config_from_parts(
            None,
            None,
            Some("[cache\nmax_size = 1"),
            PathBuf::from("/tmp/default"),
        )
        .expect_err("invalid toml should fail");
        assert_invalid_argument_contains(err, &["cache"]);
    }

    #[test]
    fn unknown_toml_field_returns_error() {
        let err = resolve_cache_config_from_parts(
            None,
            None,
            Some("[cache]\nunknown = 1\n"),
            PathBuf::from("/tmp/default"),
        )
        .expect_err("unknown field should fail");
        assert_invalid_argument_contains(err, &["unknown"]);
    }

    #[test]
    fn toml_zero_max_size_returns_error() {
        let err = resolve_cache_config_from_parts(
            None,
            None,
            Some("[cache]\nmax_size = 0\n"),
            PathBuf::from("/tmp/default"),
        )
        .expect_err("zero max_size should fail");
        assert_invalid_argument_contains(err, &["max_size", "greater than 0"]);
    }

    #[test]
    fn toml_zero_max_age_returns_error() {
        let err = resolve_cache_config_from_parts(
            None,
            None,
            Some("[cache]\nmax_age_secs = 0\n"),
            PathBuf::from("/tmp/default"),
        )
        .expect_err("zero max_age should fail");
        assert_invalid_argument_contains(err, &["max_age_secs", "greater than 0"]);
    }

    #[test]
    fn blank_toml_dir_returns_error() {
        let err = resolve_cache_config_from_parts(
            None,
            None,
            Some("[cache]\ndir = \"   \"\n"),
            PathBuf::from("/tmp/default"),
        )
        .expect_err("blank dir should fail");
        assert_invalid_argument_contains(err, &["dir"]);
    }

    #[test]
    fn toml_without_cache_section_uses_defaults() {
        let default_root = PathBuf::from("/tmp/default-cache");
        let config = resolve_cache_config_from_parts(
            None,
            None,
            Some("# no [cache] section\n"),
            default_root.clone(),
        )
        .expect("missing [cache] should use defaults");
        assert_eq!(config, default_config_with_root(default_root));
    }

    #[test]
    fn blank_env_values_are_treated_as_unset() {
        let config = resolve_cache_config_from_parts(
            Some("   "),
            Some("   "),
            Some("[cache]\ndir = \"/file-cache\"\nmax_size = 42\n"),
            PathBuf::from("/tmp/default"),
        )
        .expect("blank env values should fall through");
        assert_eq!(config.cache_root, PathBuf::from("/file-cache"));
        assert_eq!(config.max_size, 42);
    }

    #[test]
    fn resolve_cache_config_uses_defaults_when_no_env_or_file() {
        let default_root = PathBuf::from("/tmp/default-cache");
        let config = resolve_cache_config_with_source(
            None,
            None,
            None,
            None,
            None,
            default_root.clone(),
            None,
        )
        .expect("defaults should resolve");
        assert_eq!(config, default_config_with_root(default_root));
    }

    #[test]
    fn resolve_cache_config_reads_cache_toml_from_xdg_config_home() {
        let root = TempDirGuard::new("toml");
        let config_path = root.path().join("config-home/biomcp/cache.toml");
        let resolved_cache = root.path().join("resolved-cache");
        let toml_content = format!(
            "[cache]\ndir = \"{}\"\nmax_size = 1234\nmax_age_secs = 7200\n",
            resolved_cache.display()
        );

        let config = resolve_cache_config_with_source(
            None,
            None,
            None,
            None,
            Some(&toml_content),
            root.path().join("default-cache"),
            Some(&config_path),
        )
        .expect("toml should resolve");
        assert_eq!(config.cache_root, resolved_cache);
        assert_eq!(config.max_size, 1_234);
        assert_eq!(config.min_disk_free, DEFAULT_MIN_DISK_FREE);
        assert_eq!(config.max_age, Duration::from_secs(7_200));
        assert_eq!(
            config.origins,
            CacheConfigOrigins {
                cache_root: ConfigOrigin::File,
                max_size: ConfigOrigin::File,
                min_disk_free: ConfigOrigin::Default,
                max_age: ConfigOrigin::File,
            }
        );
    }

    #[test]
    fn resolve_cache_config_env_overrides_file() {
        let root = TempDirGuard::new("env-overrides");
        let env_dir = root.path().join("env-cache");
        let env_dir_value = format!("  {}  ", env_dir.display());

        let config = resolve_cache_config_with_source(
            Some(&env_dir_value),
            Some(" 5000 "),
            None,
            None,
            Some("[cache]\ndir = \"/file-cache\"\nmax_size = 1234\nmax_age_secs = 7200\n"),
            root.path().join("default-cache"),
            Some(&root.path().join("config-home/biomcp/cache.toml")),
        )
        .expect("env should override file");
        assert_eq!(config.cache_root, env_dir);
        assert_eq!(config.max_size, 5_000);
        assert_eq!(config.min_disk_free, DEFAULT_MIN_DISK_FREE);
        assert_eq!(config.max_age, Duration::from_secs(7_200));
        assert_eq!(
            config.origins,
            CacheConfigOrigins {
                cache_root: ConfigOrigin::Env,
                max_size: ConfigOrigin::Env,
                min_disk_free: ConfigOrigin::Default,
                max_age: ConfigOrigin::File,
            }
        );
    }

    #[test]
    fn resolve_cache_config_reports_path_on_failure() {
        let root = TempDirGuard::new("parse-error");
        let config_path = root.path().join("config-home/biomcp/cache.toml");

        let err = resolve_cache_config_with_source(
            None,
            None,
            None,
            None,
            Some("[cache\nmax_size = 1\n"),
            root.path().join("default-cache"),
            Some(&config_path),
        )
        .expect_err("invalid file should fail");
        let message = err.to_string();
        assert!(
            message.contains(&*config_path.to_string_lossy()),
            "expected parse error to include config path, got: {message}"
        );
    }

    #[test]
    fn resolve_cache_config_reports_path_on_read_failure() {
        let root = TempDirGuard::new("read-error");
        let config_dir = root.path().join("config-home/biomcp");
        std::fs::create_dir_all(&config_dir).expect("create config dir");
        let config_path = config_dir.join("cache.toml");
        std::fs::create_dir_all(&config_path).expect("create directory at cache.toml path");

        let err =
            read_cache_toml(&config_path).expect_err("directory at cache.toml path should fail");
        let message = err.to_string();
        assert!(matches!(err, BioMcpError::Io(_)));
        assert!(
            message.contains(&*config_path.to_string_lossy()),
            "expected read error to include config path, got: {message}"
        );
    }

    #[test]
    fn default_min_disk_free_is_10_percent_with_default_origin() {
        let config =
            resolve_cache_config_from_parts(None, None, None, PathBuf::from("/tmp/default-cache"))
                .expect("defaults should resolve");

        assert_eq!(config.min_disk_free, DiskFreeThreshold::Percent(10));
        assert_eq!(config.origins.min_disk_free, ConfigOrigin::Default);
    }

    #[test]
    fn toml_min_disk_free_parses_absolute_bytes_and_tracks_file_origin() {
        let config = resolve_cache_config_from_parts(
            None,
            None,
            Some("[cache]\nmin_disk_free = \"5G\"\n"),
            PathBuf::from("/tmp/default-cache"),
        )
        .expect("toml min_disk_free should resolve");

        assert_eq!(
            config.min_disk_free,
            DiskFreeThreshold::Bytes(bytesize::ByteSize::from_str("5G").expect("parse").as_u64())
        );
        assert_eq!(config.origins.min_disk_free, ConfigOrigin::File);
    }

    #[test]
    fn env_min_disk_free_overrides_file_and_tracks_env_origin() {
        let config = resolve_cache_config_with_source(
            None,
            None,
            Some("10%"),
            None,
            Some("[cache]\nmin_disk_free = \"5G\"\n"),
            PathBuf::from("/tmp/default-cache"),
            None,
        )
        .expect("env min_disk_free should override file");

        assert_eq!(config.min_disk_free, DiskFreeThreshold::Percent(10));
        assert_eq!(config.origins.min_disk_free, ConfigOrigin::Env);
    }

    #[test]
    fn invalid_env_min_disk_free_returns_error() {
        let err = resolve_cache_config_with_source(
            None,
            None,
            Some("0%"),
            None,
            None,
            PathBuf::from("/tmp/default-cache"),
            None,
        )
        .expect_err("invalid env min_disk_free should fail");
        assert_invalid_argument_contains(err, &["BIOMCP_CACHE_MIN_DISK_FREE", "greater than 0"]);
    }

    #[test]
    fn invalid_env_min_disk_free_over_100_percent_returns_error() {
        let err = resolve_cache_config_with_source(
            None,
            None,
            Some("101%"),
            None,
            None,
            PathBuf::from("/tmp/default-cache"),
            None,
        )
        .expect_err("percent > 100 should fail");
        assert_invalid_argument_contains(err, &["BIOMCP_CACHE_MIN_DISK_FREE", "between 1 and 100"]);
    }

    #[test]
    fn disk_free_threshold_methods_cover_percent_bytes_and_display() {
        let percent = DiskFreeThreshold::Percent(10);
        assert_eq!(percent.required_free_bytes(1_000), 100);
        assert!(percent.is_violated(99, 1_000));
        assert_eq!(percent.display(), "10%");

        let bytes = DiskFreeThreshold::Bytes(500);
        assert_eq!(bytes.required_free_bytes(1_000), 500);
        assert!(bytes.is_violated(499, 1_000));
        assert_eq!(bytes.display(), "500 B");
    }
}
