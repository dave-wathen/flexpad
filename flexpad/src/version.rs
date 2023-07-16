pub struct Version {
    pub version: &'static str,
    pub git_branch: &'static str,
    pub git_sha: &'static str,
}

impl Default for Version {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
            git_branch: env!("VERGEN_GIT_BRANCH"),
            git_sha: env!("VERGEN_GIT_SHA"),
        }
    }
}

impl Version {
    pub fn description(&self) -> String {
        if self.git_branch == self.version {
            format!("Flexpad {} ({})", self.version, &self.git_sha[0..7])
        } else {
            format!(
                "Flexpad {} ({} {})",
                self.version,
                self.git_branch,
                &self.git_sha[0..7]
            )
        }
    }
}
