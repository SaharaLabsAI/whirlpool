#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FullDkgFeatureGate {
    enabled: bool,
}

impl Default for FullDkgFeatureGate {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl FullDkgFeatureGate {
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}
