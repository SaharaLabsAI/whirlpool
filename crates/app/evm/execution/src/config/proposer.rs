#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ProposerRuntimeContext {
    local_public_key: [u8; 32],
}

impl ProposerRuntimeContext {
    pub fn with_local_public_key(mut self, local_public_key: [u8; 32]) -> Self {
        self.local_public_key = local_public_key;
        self
    }

    pub fn local_public_key(&self) -> [u8; 32] {
        self.local_public_key
    }
}
