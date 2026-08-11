use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildIdentity {
    pub version: &'static str,
    pub git_revision: &'static str,
    pub build_date: &'static str,
}

static BUILD_IDENTITY: OnceLock<BuildIdentity> = OnceLock::new();

const PACKAGE_IDENTITY: BuildIdentity = BuildIdentity {
    version: env!("CARGO_PKG_VERSION"),
    git_revision: "unknown",
    build_date: "unknown",
};

pub fn install(identity: BuildIdentity) {
    let _ = BUILD_IDENTITY.set(identity);
}

pub fn current() -> BuildIdentity {
    BUILD_IDENTITY.get().copied().unwrap_or(PACKAGE_IDENTITY)
}
