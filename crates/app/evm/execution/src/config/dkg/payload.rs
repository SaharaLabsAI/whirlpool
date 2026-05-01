use validators_dkg::FullDkgOutputV1;

#[derive(Debug, Clone, Default)]
pub struct CurrentFullDkgCandidate {
    output: Option<FullDkgOutputV1>,
}

impl CurrentFullDkgCandidate {
    pub fn with_output(mut self, output: FullDkgOutputV1) -> Self {
        self.output = Some(output);
        self
    }

    pub fn output(&self) -> Option<&FullDkgOutputV1> {
        self.output.as_ref()
    }
}
