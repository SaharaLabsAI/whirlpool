#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullDkgOutputV1 {
    pub dealers: Vec<[u8; 32]>,
    pub players: Vec<[u8; 32]>,
    pub public_polynomial: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullDkgV1 {
    pub epoch: u64,
    pub output: FullDkgOutputV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReshareV1 {
    pub target_epoch: u64,
    pub players: Vec<[u8; 32]>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct DkgHeaderDecision {
    pub full_dkg: Option<FullDkgV1>,
    pub reshare: Option<ReshareV1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DkgHeaderSectionsRef<'a> {
    pub full_dkg: Option<&'a FullDkgV1>,
    pub reshare: Option<&'a ReshareV1>,
}
