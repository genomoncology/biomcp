//! Managed skill payload hashing and read-only installation status classification.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

use crate::error::BioMcpError;

use super::assets::canonical_prompt_file_bytes;
use super::install::resolve_skill_target;

pub(super) const MANIFEST_NAME: &str = ".biomcp-skill.json";
const MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub(super) struct ManagedPayload {
    pub files: BTreeMap<String, Vec<u8>>,
    pub manifest: SkillManifest,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct SkillManifest {
    pub schema_version: u32,
    pub biomcp_version: String,
    pub render_sha256: String,
    pub installed_at: String,
    #[serde(deserialize_with = "deserialize_managed_files")]
    pub managed_files: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum SkillState {
    Missing,
    Unmanaged,
    LocallyModified,
    Stale,
    Current,
}

impl SkillState {
    fn label(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Unmanaged => "unmanaged",
            Self::LocallyModified => "locally_modified",
            Self::Stale => "stale",
            Self::Current => "current",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillStatus {
    state: SkillState,
    installed_version: Option<String>,
    installed_render_sha256: Option<String>,
    current_version: String,
    current_render_sha256: String,
    recovery_command: Option<String>,
}

struct ManagedFilesVisitor;

impl<'de> Visitor<'de> for ManagedFilesVisitor {
    type Value = BTreeMap<String, String>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a map of unique managed paths to SHA-256 digests")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut files = BTreeMap::new();
        while let Some((path, digest)) = map.next_entry::<String, String>()? {
            if files.insert(path.clone(), digest).is_some() {
                return Err(serde::de::Error::custom(format!(
                    "duplicate managed path {path:?}"
                )));
            }
        }
        Ok(files)
    }
}

fn deserialize_managed_files<'de, D>(deserializer: D) -> Result<BTreeMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_map(ManagedFilesVisitor)
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn validate_managed_path(path: &str) -> Result<PathBuf, BioMcpError> {
    if path.is_empty() || path == MANIFEST_NAME || path.contains('\\') {
        return Err(BioMcpError::InvalidArgument(
            "Invalid managed skill path".into(),
        ));
    }
    let parsed = Path::new(path);
    if parsed.is_absolute()
        || parsed
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(BioMcpError::InvalidArgument(
            "Invalid managed skill path".into(),
        ));
    }
    let normalized = parsed
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    if normalized != path {
        return Err(BioMcpError::InvalidArgument(
            "Invalid managed skill path".into(),
        ));
    }
    Ok(parsed.to_path_buf())
}

pub(super) fn current_payload() -> Result<ManagedPayload, BioMcpError> {
    let mut files = BTreeMap::new();
    for embedded in crate::skill_assets::iter() {
        let path = embedded.as_ref();
        validate_managed_path(path)?;
        let asset = crate::skill_assets::bytes(path)?;
        let bytes = if path == "SKILL.md" {
            canonical_prompt_file_bytes()?
        } else {
            asset.into_owned()
        };
        if files.insert(path.to_string(), bytes).is_some() {
            return Err(BioMcpError::InvalidArgument(
                "Duplicate embedded skill path".into(),
            ));
        }
    }

    let render = files.get("SKILL.md").ok_or_else(|| {
        BioMcpError::InvalidArgument("Embedded skill payload is missing SKILL.md".into())
    })?;
    let managed_files = files
        .iter()
        .map(|(path, bytes)| (path.clone(), sha256_hex(bytes)))
        .collect();
    let manifest = SkillManifest {
        schema_version: MANIFEST_SCHEMA_VERSION,
        biomcp_version: env!("CARGO_PKG_VERSION").to_string(),
        render_sha256: sha256_hex(render),
        installed_at: chrono::Utc::now().to_rfc3339(),
        managed_files,
    };
    Ok(ManagedPayload { files, manifest })
}

pub(super) fn parse_valid_manifest(target: &Path) -> Result<Option<SkillManifest>, BioMcpError> {
    let path = target.join(MANIFEST_NAME);
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {}
        Ok(_) => return Ok(None),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(BioMcpError::Io(error)),
    }
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(BioMcpError::Io(error)),
    };
    let Ok(manifest) = serde_json::from_slice::<SkillManifest>(&bytes) else {
        return Ok(None);
    };
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        return Ok(None);
    }
    for path in manifest.managed_files.keys() {
        if validate_managed_path(path).is_err() {
            return Ok(None);
        }
    }
    Ok(Some(manifest))
}

fn read_recorded_file(target: &Path, relative: &str) -> Result<Option<Vec<u8>>, BioMcpError> {
    let relative = validate_managed_path(relative)?;
    let mut cursor = target.to_path_buf();
    for component in relative.components() {
        cursor.push(component.as_os_str());
        let metadata = match fs::symlink_metadata(&cursor) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(BioMcpError::Io(error)),
        };
        if metadata.file_type().is_symlink() {
            return Err(BioMcpError::InvalidArgument(
                "Managed skill path contains a symbolic link".into(),
            ));
        }
    }
    let metadata = fs::metadata(&cursor)?;
    if !metadata.is_file() {
        return Ok(None);
    }
    fs::read(cursor).map(Some).map_err(BioMcpError::Io)
}

pub(super) fn classify_target(
    target: &Path,
    safe_dir: &str,
    current: &ManagedPayload,
) -> Result<SkillStatus, BioMcpError> {
    let recovery = || format!("biomcp skill install --force {safe_dir}");
    let current_version = current.manifest.biomcp_version.clone();
    let current_render_sha256 = current.manifest.render_sha256.clone();
    let report = |state, manifest: Option<&SkillManifest>| SkillStatus {
        state,
        installed_version: manifest.map(|value| value.biomcp_version.clone()),
        installed_render_sha256: manifest.map(|value| value.render_sha256.clone()),
        current_version: current_version.clone(),
        current_render_sha256: current_render_sha256.clone(),
        recovery_command: (state != SkillState::Current).then(recovery),
    };

    let target_metadata = match fs::symlink_metadata(target) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(report(SkillState::Missing, None));
        }
        Err(error) => return Err(BioMcpError::Io(error)),
    };
    if target_metadata.file_type().is_symlink() || !target_metadata.is_dir() {
        return Err(BioMcpError::InvalidArgument(
            "Skill target must be a real directory".into(),
        ));
    }
    match fs::symlink_metadata(target.join("SKILL.md")) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {}
        Ok(_) => return Ok(report(SkillState::Missing, None)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(report(SkillState::Missing, None));
        }
        Err(error) => return Err(BioMcpError::Io(error)),
    }

    let Some(manifest) = parse_valid_manifest(target)? else {
        return Ok(report(SkillState::Unmanaged, None));
    };
    for (path, expected_digest) in &manifest.managed_files {
        let Some(bytes) = read_recorded_file(target, path)? else {
            return Ok(report(SkillState::LocallyModified, Some(&manifest)));
        };
        if sha256_hex(&bytes) != *expected_digest {
            return Ok(report(SkillState::LocallyModified, Some(&manifest)));
        }
    }

    if manifest.biomcp_version != current.manifest.biomcp_version
        || manifest.render_sha256 != current.manifest.render_sha256
        || manifest.managed_files != current.manifest.managed_files
    {
        return Ok(report(SkillState::Stale, Some(&manifest)));
    }
    Ok(report(SkillState::Current, Some(&manifest)))
}

fn render_markdown(status: &SkillStatus) -> String {
    let mut output = format!("# BioMCP Skill Status\n\nState: {}", status.state.label());
    if let Some(version) = &status.installed_version {
        let _ = write!(output, "\nInstalled version: {version}");
    }
    let _ = write!(output, "\nCurrent version: {}", status.current_version);
    if let Some(command) = &status.recovery_command {
        let _ = write!(output, "\nRecovery: `{command}`");
    }
    output
}

/// Reports whether an installed managed skill matches this BioMCP binary.
///
/// # Errors
///
/// Returns an error when target resolution, embedded payload validation, or a
/// required filesystem read fails.
pub fn skill_status(dir: Option<&str>, json: bool) -> Result<String, BioMcpError> {
    let resolved = resolve_skill_target(dir)?;
    let current = current_payload()?;
    let status = classify_target(&resolved.path, &resolved.safe_dir, &current)?;
    if json {
        crate::render::json::to_pretty(&status)
    } else {
        Ok(render_markdown(&status))
    }
}
