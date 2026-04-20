use bytes::{Buf, BufMut};
use commonware_codec::{EncodeSize, Error as CodecError, Read as CodecRead, Write as CodecWrite};
use commonware_consensus::{Block as VendorBlock, Heightable};
use commonware_cryptography::{sha256, Committable, Digestible};
use consensus::traits::Block as CoreBlock;
use sha2::{Digest as Sha2Digest, Sha256};
use std::fmt;

pub type BlockId = [u8; 32];

type BlockDigest = sha256::Digest;

const EXTRA_DATA_MAGIC: &[u8; 4] = b"WDX1";
const EXTRA_DATA_VERSION: u8 = 1;
const EXTRA_DATA_SECTION_RAW_ETH: u8 = 1;
const EXTRA_DATA_SECTION_FULL_DKG_V1: u8 = 2;
const EXTRA_DATA_SECTION_RESHARE_V1: u8 = 3;
const MAX_TOTAL_EXTRA_DATA_BYTES: usize = 256 * 1024;
const MAX_RAW_ETH_EXTRA_DATA_BYTES: usize = 1024;
const MAX_FULL_DKG_KEYS: usize = 1024;
const MAX_RESHARE_KEYS: usize = 1024;
const MAX_FULL_DKG_POLYNOMIAL_BYTES: usize = 128 * 1024;
pub const LEGACY_PROPOSER_EXTRA_DATA_LEN: usize = 32;

#[derive(Clone, Debug)]
pub struct ExecutionResult {
    pub state_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub gas_used: u64,
    pub receipt_count: usize,
}

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
pub struct CanonicalExtraDataV1 {
    pub raw_eth: Option<Vec<u8>>,
    pub full_dkg: Option<FullDkgV1>,
    pub reshare: Option<ReshareV1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtraDataDecodeMode {
    Strict,
    Legacy,
    RpcProjection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExtraDataError {
    EmptySections,
    InvalidMagic,
    UnsupportedVersion {
        found: u8,
    },
    UnexpectedTrailingBytes,
    Truncated(&'static str),
    DuplicateSection {
        section: u8,
    },
    InvalidSectionOrder {
        section: u8,
    },
    UnknownSection {
        section: u8,
    },
    RawEthTooLarge {
        found: usize,
        max: usize,
    },
    InvalidRawEthLen {
        found: usize,
    },
    LegacyUnsupportedLen {
        found: usize,
    },
    TooManyDealers {
        found: usize,
        max: usize,
    },
    TooManyPlayers {
        found: usize,
        max: usize,
    },
    TooManyResharePlayers {
        found: usize,
        max: usize,
    },
    FullDkgPolynomialTooLarge {
        found: usize,
        max: usize,
    },
    TotalExtraDataTooLarge {
        found: usize,
        max: usize,
    },
    SectionTooLarge {
        section: u8,
        found: usize,
        max: usize,
    },
}

impl fmt::Display for ExtraDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySections => write!(f, "canonical extra_data must contain at least one section"),
            Self::InvalidMagic => write!(f, "invalid canonical extra_data magic"),
            Self::UnsupportedVersion { found } => {
                write!(f, "unsupported canonical extra_data version: {found}")
            }
            Self::UnexpectedTrailingBytes => write!(f, "unexpected trailing bytes in canonical extra_data"),
            Self::Truncated(ctx) => write!(f, "truncated canonical extra_data while reading {ctx}"),
            Self::DuplicateSection { section } => {
                write!(f, "duplicate canonical extra_data section: {section}")
            }
            Self::InvalidSectionOrder { section } => {
                write!(f, "invalid canonical extra_data section order at section: {section}")
            }
            Self::UnknownSection { section } => {
                write!(f, "unknown canonical extra_data section: {section}")
            }
            Self::RawEthTooLarge { found, max } => {
                write!(f, "raw_eth section too large: found {found}, max {max}")
            }
            Self::InvalidRawEthLen { found } => write!(
                f,
                "raw_eth proposer key must be {LEGACY_PROPOSER_EXTRA_DATA_LEN} bytes, found {found}"
            ),
            Self::LegacyUnsupportedLen { found } => write!(
                f,
                "legacy extra_data must be exactly {LEGACY_PROPOSER_EXTRA_DATA_LEN} bytes, found {found}"
            ),
            Self::TooManyDealers { found, max } => {
                write!(f, "too many full_dkg dealers: found {found}, max {max}")
            }
            Self::TooManyPlayers { found, max } => {
                write!(f, "too many full_dkg players: found {found}, max {max}")
            }
            Self::TooManyResharePlayers { found, max } => {
                write!(f, "too many reshare players: found {found}, max {max}")
            }
            Self::FullDkgPolynomialTooLarge { found, max } => {
                write!(f, "full_dkg public_polynomial too large: found {found}, max {max}")
            }
            Self::TotalExtraDataTooLarge { found, max } => {
                write!(f, "canonical extra_data too large: found {found}, max {max}")
            }
            Self::SectionTooLarge {
                section,
                found,
                max,
            } => {
                write!(f, "section {section} too large: found {found}, max {max}")
            }
        }
    }
}

impl std::error::Error for ExtraDataError {}

#[derive(Clone, Debug)]
pub struct EvmBlock {
    pub height: u64,
    pub parent_id: [u8; 32],
    pub state_root: [u8; 32],
    pub transactions_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub proposer_public_key: [u8; 32],
    pub proposer_fee_recipient: [u8; 20],
    pub extra_data: Vec<u8>,
    pub gas_used: u64,
    pub base_fee_per_gas: u64,
    pub timestamp: u64,
    pub transactions: Vec<Vec<u8>>,
}

impl EvmBlock {
    pub fn compute_id(&self) -> BlockId {
        let mut hasher = Sha256::new();
        hasher.update(self.height.to_le_bytes());
        hasher.update(self.parent_id);
        hasher.update(self.state_root);
        hasher.update(self.transactions_root);
        hasher.update(self.proposer_public_key);
        hasher.update(self.proposer_fee_recipient);
        hasher.update((self.extra_data.len() as u32).to_le_bytes());
        hasher.update(&self.extra_data);

        let result = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&result);
        id
    }

    fn compute_digest(&self) -> BlockDigest {
        let mut hasher = Sha256::new();
        hasher.update(self.height.to_le_bytes());
        hasher.update(self.parent_id);
        hasher.update(self.state_root);
        hasher.update(self.transactions_root);
        hasher.update(self.receipts_root);
        hasher.update(self.proposer_public_key);
        hasher.update(self.proposer_fee_recipient);
        hasher.update((self.extra_data.len() as u32).to_le_bytes());
        hasher.update(&self.extra_data);
        hasher.update(self.gas_used.to_le_bytes());
        hasher.update(self.base_fee_per_gas.to_le_bytes());
        hasher.update(self.timestamp.to_le_bytes());
        hasher.update((self.transactions.len() as u32).to_le_bytes());
        for tx in &self.transactions {
            hasher.update((tx.len() as u32).to_le_bytes());
            hasher.update(tx);
        }

        let digest = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&digest);
        BlockDigest::from(bytes)
    }
}

pub fn legacy_proposer_extra_data_bytes(proposer_public_key: [u8; 32]) -> Vec<u8> {
    proposer_public_key.to_vec()
}

pub fn proposer_public_key_from_extra_data(extra_data: &[u8]) -> Option<[u8; 32]> {
    let decoded = decode_extra_data(extra_data, ExtraDataDecodeMode::Legacy).ok()?;
    let raw_eth = decoded.raw_eth?;
    if raw_eth.len() != LEGACY_PROPOSER_EXTRA_DATA_LEN {
        return None;
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&raw_eth);
    Some(out)
}

pub fn project_raw_eth_extra_data(extra_data: &[u8]) -> Vec<u8> {
    match decode_extra_data(extra_data, ExtraDataDecodeMode::RpcProjection) {
        Ok(decoded) => decoded.raw_eth.unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn encode_canonical_extra_data(data: &CanonicalExtraDataV1) -> Result<Vec<u8>, ExtraDataError> {
    let mut sections: Vec<(u8, Vec<u8>)> = Vec::new();

    if let Some(raw_eth) = &data.raw_eth {
        if raw_eth.len() > MAX_RAW_ETH_EXTRA_DATA_BYTES {
            return Err(ExtraDataError::RawEthTooLarge {
                found: raw_eth.len(),
                max: MAX_RAW_ETH_EXTRA_DATA_BYTES,
            });
        }
        sections.push((EXTRA_DATA_SECTION_RAW_ETH, raw_eth.clone()));
    }

    if let Some(full_dkg) = &data.full_dkg {
        let full_dkg_payload = encode_full_dkg_v1(full_dkg)?;
        sections.push((EXTRA_DATA_SECTION_FULL_DKG_V1, full_dkg_payload));
    }

    if let Some(reshare) = &data.reshare {
        if data.full_dkg.is_none() {
            return Err(ExtraDataError::InvalidSectionOrder {
                section: EXTRA_DATA_SECTION_RESHARE_V1,
            });
        }
        let reshare_payload = encode_reshare_v1(reshare)?;
        sections.push((EXTRA_DATA_SECTION_RESHARE_V1, reshare_payload));
    }

    if sections.is_empty() {
        return Err(ExtraDataError::EmptySections);
    }

    let mut out = Vec::new();
    out.extend_from_slice(EXTRA_DATA_MAGIC);
    out.push(EXTRA_DATA_VERSION);
    out.push(u8::try_from(sections.len()).expect("section count fits into u8"));

    for (section, payload) in sections {
        out.push(section);
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&payload);
    }

    if out.len() > MAX_TOTAL_EXTRA_DATA_BYTES {
        return Err(ExtraDataError::TotalExtraDataTooLarge {
            found: out.len(),
            max: MAX_TOTAL_EXTRA_DATA_BYTES,
        });
    }

    Ok(out)
}

pub fn decode_extra_data(
    bytes: &[u8],
    mode: ExtraDataDecodeMode,
) -> Result<CanonicalExtraDataV1, ExtraDataError> {
    if bytes.starts_with(EXTRA_DATA_MAGIC) {
        return decode_enveloped_extra_data(bytes);
    }

    match mode {
        ExtraDataDecodeMode::Strict => Err(ExtraDataError::InvalidMagic),
        ExtraDataDecodeMode::Legacy | ExtraDataDecodeMode::RpcProjection => {
            if bytes.len() != LEGACY_PROPOSER_EXTRA_DATA_LEN {
                return Err(ExtraDataError::LegacyUnsupportedLen { found: bytes.len() });
            }
            Ok(CanonicalExtraDataV1 {
                raw_eth: Some(bytes.to_vec()),
                full_dkg: None,
                reshare: None,
            })
        }
    }
}

fn decode_enveloped_extra_data(bytes: &[u8]) -> Result<CanonicalExtraDataV1, ExtraDataError> {
    if bytes.len() > MAX_TOTAL_EXTRA_DATA_BYTES {
        return Err(ExtraDataError::TotalExtraDataTooLarge {
            found: bytes.len(),
            max: MAX_TOTAL_EXTRA_DATA_BYTES,
        });
    }

    let mut cursor = bytes;
    let magic = take_slice(&mut cursor, 4, "magic")?;
    if magic != EXTRA_DATA_MAGIC {
        return Err(ExtraDataError::InvalidMagic);
    }

    let version = take_u8(&mut cursor, "version")?;
    if version != EXTRA_DATA_VERSION {
        return Err(ExtraDataError::UnsupportedVersion { found: version });
    }

    let section_count = take_u8(&mut cursor, "section_count")? as usize;
    let mut raw_eth = None;
    let mut full_dkg = None;
    let mut reshare = None;

    for _ in 0..section_count {
        let section = take_u8(&mut cursor, "section id")?;
        let section_len = take_u32(&mut cursor, "section length")? as usize;

        if section_len > MAX_TOTAL_EXTRA_DATA_BYTES {
            return Err(ExtraDataError::SectionTooLarge {
                section,
                found: section_len,
                max: MAX_TOTAL_EXTRA_DATA_BYTES,
            });
        }

        let payload = take_slice(&mut cursor, section_len, "section payload")?;

        match section {
            EXTRA_DATA_SECTION_RAW_ETH => {
                if full_dkg.is_some() {
                    return Err(ExtraDataError::InvalidSectionOrder { section });
                }
                if raw_eth.is_some() {
                    return Err(ExtraDataError::DuplicateSection { section });
                }
                if payload.len() > MAX_RAW_ETH_EXTRA_DATA_BYTES {
                    return Err(ExtraDataError::RawEthTooLarge {
                        found: payload.len(),
                        max: MAX_RAW_ETH_EXTRA_DATA_BYTES,
                    });
                }
                raw_eth = Some(payload.to_vec());
            }
            EXTRA_DATA_SECTION_FULL_DKG_V1 => {
                if full_dkg.is_some() {
                    return Err(ExtraDataError::DuplicateSection { section });
                }
                if reshare.is_some() {
                    return Err(ExtraDataError::InvalidSectionOrder { section });
                }
                full_dkg = Some(decode_full_dkg_v1(payload)?);
            }
            EXTRA_DATA_SECTION_RESHARE_V1 => {
                if reshare.is_some() {
                    return Err(ExtraDataError::DuplicateSection { section });
                }
                if full_dkg.is_none() {
                    return Err(ExtraDataError::InvalidSectionOrder { section });
                }
                reshare = Some(decode_reshare_v1(payload)?);
            }
            _ => return Err(ExtraDataError::UnknownSection { section }),
        }
    }

    if !cursor.is_empty() {
        return Err(ExtraDataError::UnexpectedTrailingBytes);
    }

    if raw_eth.is_none() && full_dkg.is_none() && reshare.is_none() {
        return Err(ExtraDataError::EmptySections);
    }

    Ok(CanonicalExtraDataV1 {
        raw_eth,
        full_dkg,
        reshare,
    })
}

fn encode_full_dkg_v1(full_dkg: &FullDkgV1) -> Result<Vec<u8>, ExtraDataError> {
    if full_dkg.output.dealers.len() > MAX_FULL_DKG_KEYS {
        return Err(ExtraDataError::TooManyDealers {
            found: full_dkg.output.dealers.len(),
            max: MAX_FULL_DKG_KEYS,
        });
    }
    if full_dkg.output.players.len() > MAX_FULL_DKG_KEYS {
        return Err(ExtraDataError::TooManyPlayers {
            found: full_dkg.output.players.len(),
            max: MAX_FULL_DKG_KEYS,
        });
    }
    if full_dkg.output.public_polynomial.len() > MAX_FULL_DKG_POLYNOMIAL_BYTES {
        return Err(ExtraDataError::FullDkgPolynomialTooLarge {
            found: full_dkg.output.public_polynomial.len(),
            max: MAX_FULL_DKG_POLYNOMIAL_BYTES,
        });
    }

    let mut out = Vec::new();
    out.extend_from_slice(&full_dkg.epoch.to_le_bytes());
    out.extend_from_slice(&(full_dkg.output.dealers.len() as u32).to_le_bytes());
    for dealer in &full_dkg.output.dealers {
        out.extend_from_slice(dealer);
    }
    out.extend_from_slice(&(full_dkg.output.players.len() as u32).to_le_bytes());
    for player in &full_dkg.output.players {
        out.extend_from_slice(player);
    }
    out.extend_from_slice(&(full_dkg.output.public_polynomial.len() as u32).to_le_bytes());
    out.extend_from_slice(&full_dkg.output.public_polynomial);
    Ok(out)
}

fn decode_full_dkg_v1(bytes: &[u8]) -> Result<FullDkgV1, ExtraDataError> {
    let mut cursor = bytes;
    let epoch = take_u64(&mut cursor, "full_dkg.epoch")?;

    let dealers_len = take_u32(&mut cursor, "full_dkg.dealers_len")? as usize;
    if dealers_len > MAX_FULL_DKG_KEYS {
        return Err(ExtraDataError::TooManyDealers {
            found: dealers_len,
            max: MAX_FULL_DKG_KEYS,
        });
    }

    let mut dealers = Vec::with_capacity(dealers_len);
    for _ in 0..dealers_len {
        let dealer = take_slice(&mut cursor, 32, "full_dkg.dealer")?;
        let mut dealer_key = [0u8; 32];
        dealer_key.copy_from_slice(dealer);
        dealers.push(dealer_key);
    }

    let players_len = take_u32(&mut cursor, "full_dkg.players_len")? as usize;
    if players_len > MAX_FULL_DKG_KEYS {
        return Err(ExtraDataError::TooManyPlayers {
            found: players_len,
            max: MAX_FULL_DKG_KEYS,
        });
    }

    let mut players = Vec::with_capacity(players_len);
    for _ in 0..players_len {
        let player = take_slice(&mut cursor, 32, "full_dkg.player")?;
        let mut player_key = [0u8; 32];
        player_key.copy_from_slice(player);
        players.push(player_key);
    }

    let polynomial_len = take_u32(&mut cursor, "full_dkg.public_polynomial_len")? as usize;
    if polynomial_len > MAX_FULL_DKG_POLYNOMIAL_BYTES {
        return Err(ExtraDataError::FullDkgPolynomialTooLarge {
            found: polynomial_len,
            max: MAX_FULL_DKG_POLYNOMIAL_BYTES,
        });
    }
    let public_polynomial =
        take_slice(&mut cursor, polynomial_len, "full_dkg.public_polynomial")?.to_vec();

    if !cursor.is_empty() {
        return Err(ExtraDataError::UnexpectedTrailingBytes);
    }

    Ok(FullDkgV1 {
        epoch,
        output: FullDkgOutputV1 {
            dealers,
            players,
            public_polynomial,
        },
    })
}

fn encode_reshare_v1(reshare: &ReshareV1) -> Result<Vec<u8>, ExtraDataError> {
    if reshare.players.len() > MAX_RESHARE_KEYS {
        return Err(ExtraDataError::TooManyResharePlayers {
            found: reshare.players.len(),
            max: MAX_RESHARE_KEYS,
        });
    }

    let mut out = Vec::new();
    out.extend_from_slice(&reshare.target_epoch.to_le_bytes());
    out.extend_from_slice(&(reshare.players.len() as u32).to_le_bytes());
    for player in &reshare.players {
        out.extend_from_slice(player);
    }
    Ok(out)
}

fn decode_reshare_v1(bytes: &[u8]) -> Result<ReshareV1, ExtraDataError> {
    let mut cursor = bytes;
    let target_epoch = take_u64(&mut cursor, "reshare.target_epoch")?;

    let players_len = take_u32(&mut cursor, "reshare.players_len")? as usize;
    if players_len > MAX_RESHARE_KEYS {
        return Err(ExtraDataError::TooManyResharePlayers {
            found: players_len,
            max: MAX_RESHARE_KEYS,
        });
    }

    let mut players = Vec::with_capacity(players_len);
    for _ in 0..players_len {
        let player = take_slice(&mut cursor, 32, "reshare.player")?;
        let mut player_key = [0u8; 32];
        player_key.copy_from_slice(player);
        players.push(player_key);
    }

    if !cursor.is_empty() {
        return Err(ExtraDataError::UnexpectedTrailingBytes);
    }

    Ok(ReshareV1 {
        target_epoch,
        players,
    })
}

fn take_slice<'a>(
    cursor: &mut &'a [u8],
    len: usize,
    label: &'static str,
) -> Result<&'a [u8], ExtraDataError> {
    if cursor.remaining() < len {
        return Err(ExtraDataError::Truncated(label));
    }
    let out = &cursor[..len];
    *cursor = &cursor[len..];
    Ok(out)
}

fn take_u8(cursor: &mut &[u8], label: &'static str) -> Result<u8, ExtraDataError> {
    Ok(*take_slice(cursor, 1, label)?
        .first()
        .expect("slice len checked"))
}

fn take_u32(cursor: &mut &[u8], label: &'static str) -> Result<u32, ExtraDataError> {
    let bytes = take_slice(cursor, 4, label)?;
    let mut out = [0u8; 4];
    out.copy_from_slice(bytes);
    Ok(u32::from_le_bytes(out))
}

fn take_u64(cursor: &mut &[u8], label: &'static str) -> Result<u64, ExtraDataError> {
    let bytes = take_slice(cursor, 8, label)?;
    let mut out = [0u8; 8];
    out.copy_from_slice(bytes);
    Ok(u64::from_le_bytes(out))
}

impl CoreBlock for EvmBlock {
    type Id = BlockId;

    fn id(&self) -> BlockId {
        self.compute_id()
    }

    fn parent_id(&self) -> BlockId {
        self.parent_id
    }

    fn height(&self) -> u64 {
        self.height
    }
}

impl CodecWrite for EvmBlock {
    fn write(&self, buf: &mut impl BufMut) {
        buf.put_u64(self.height);
        buf.put_slice(&self.parent_id);
        buf.put_slice(&self.state_root);
        buf.put_slice(&self.transactions_root);
        buf.put_slice(&self.receipts_root);
        buf.put_slice(&self.proposer_public_key);
        buf.put_slice(&self.proposer_fee_recipient);
        buf.put_u32(self.extra_data.len() as u32);
        buf.put_slice(&self.extra_data);
        buf.put_u64(self.gas_used);
        buf.put_u64(self.base_fee_per_gas);
        buf.put_u64(self.timestamp);
        buf.put_u32(self.transactions.len() as u32);
        for tx in &self.transactions {
            buf.put_u32(tx.len() as u32);
            buf.put_slice(tx);
        }
    }
}

impl EncodeSize for EvmBlock {
    fn encode_size(&self) -> usize {
        8 + 32
            + 32
            + 32
            + 32
            + 32
            + 20
            + 4
            + self.extra_data.len()
            + 8
            + 8
            + 8
            + 4
            + self
                .transactions
                .iter()
                .map(|tx| 4 + tx.len())
                .sum::<usize>()
    }
}

impl CodecRead for EvmBlock {
    type Cfg = ();

    fn read_cfg(reader: &mut impl Buf, _cfg: &Self::Cfg) -> Result<Self, CodecError> {
        const MIN_ENCODED_BLOCK_LEN: usize = 8 + 32 + 32 + 32 + 32 + 32 + 20 + 4 + 8 + 8 + 8 + 4;
        if reader.remaining() < MIN_ENCODED_BLOCK_LEN {
            return Err(CodecError::Invalid("EvmBlock", "not enough bytes"));
        }

        let height = reader.get_u64();

        let mut parent_id = [0u8; 32];
        reader.copy_to_slice(&mut parent_id);

        let mut state_root = [0u8; 32];
        reader.copy_to_slice(&mut state_root);

        let mut transactions_root = [0u8; 32];
        reader.copy_to_slice(&mut transactions_root);

        let mut receipts_root = [0u8; 32];
        reader.copy_to_slice(&mut receipts_root);

        let mut proposer_public_key = [0u8; 32];
        reader.copy_to_slice(&mut proposer_public_key);

        let mut proposer_fee_recipient = [0u8; 20];
        reader.copy_to_slice(&mut proposer_fee_recipient);

        if reader.remaining() < 4 {
            return Err(CodecError::Invalid("EvmBlock", "missing extra_data length"));
        }
        let extra_data_len = reader.get_u32() as usize;
        if reader.remaining() < extra_data_len {
            return Err(CodecError::Invalid(
                "EvmBlock",
                "extra_data exceeds remaining bytes",
            ));
        }
        let mut extra_data = vec![0u8; extra_data_len];
        reader.copy_to_slice(&mut extra_data);

        let gas_used = reader.get_u64();
        let base_fee_per_gas = reader.get_u64();
        let timestamp = reader.get_u64();

        let tx_count = reader.get_u32() as usize;
        let mut transactions = Vec::with_capacity(tx_count);

        for _ in 0..tx_count {
            if reader.remaining() < 4 {
                return Err(CodecError::Invalid(
                    "EvmBlock",
                    "missing transaction length",
                ));
            }
            let tx_len = reader.get_u32() as usize;
            if reader.remaining() < tx_len {
                return Err(CodecError::Invalid(
                    "EvmBlock",
                    "transaction exceeds remaining bytes",
                ));
            }
            let mut tx = vec![0u8; tx_len];
            reader.copy_to_slice(&mut tx);
            transactions.push(tx);
        }

        Ok(Self {
            height,
            parent_id,
            state_root,
            transactions_root,
            receipts_root,
            proposer_public_key,
            proposer_fee_recipient,
            extra_data,
            gas_used,
            base_fee_per_gas,
            timestamp,
            transactions,
        })
    }
}

impl Digestible for EvmBlock {
    type Digest = BlockDigest;

    fn digest(&self) -> Self::Digest {
        self.compute_digest()
    }
}

impl Committable for EvmBlock {
    type Commitment = BlockDigest;

    fn commitment(&self) -> Self::Commitment {
        self.digest()
    }
}

impl Heightable for EvmBlock {
    fn height(&self) -> commonware_consensus::types::Height {
        commonware_consensus::types::Height::new(self.height)
    }
}

impl VendorBlock for EvmBlock {
    fn parent(&self) -> Self::Digest {
        BlockDigest::from(self.parent_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        decode_extra_data, encode_canonical_extra_data, legacy_proposer_extra_data_bytes,
        project_raw_eth_extra_data, CanonicalExtraDataV1, EvmBlock, ExecutionResult,
        ExtraDataDecodeMode, ExtraDataError, FullDkgOutputV1, FullDkgV1, ReshareV1,
    };
    use consensus::traits::Block as CoreBlock;

    fn sample_block() -> EvmBlock {
        EvmBlock {
            height: 10,
            parent_id: [1u8; 32],
            state_root: [2u8; 32],
            transactions_root: [3u8; 32],
            receipts_root: [4u8; 32],
            proposer_public_key: [5u8; 32],
            proposer_fee_recipient: [5u8; 20],
            extra_data: legacy_proposer_extra_data_bytes([5u8; 32]),
            gas_used: 42,
            base_fee_per_gas: 1_000_000_000,
            timestamp: 1_700_000_000,
            transactions: vec![vec![0xaa, 0xbb], vec![0xcc]],
        }
    }

    #[test]
    fn test_evm_block_trait_impl() {
        let block = sample_block();
        assert_eq!(CoreBlock::height(&block), 10);
        assert_eq!(CoreBlock::parent_id(&block), [1u8; 32]);
        assert!(CoreBlock::id(&block).iter().any(|b| *b != 0));
    }

    #[test]
    fn test_evm_block_codec_roundtrip() {
        use commonware_codec::{Read as CodecRead, Write as CodecWrite};

        let block = sample_block();
        let mut buf = bytes::BytesMut::new();
        block.write(&mut buf);
        let decoded = EvmBlock::read_cfg(&mut buf.freeze(), &()).expect("decode should succeed");

        assert_eq!(decoded.height, block.height);
        assert_eq!(decoded.parent_id, block.parent_id);
        assert_eq!(decoded.state_root, block.state_root);
        assert_eq!(decoded.transactions_root, block.transactions_root);
        assert_eq!(decoded.receipts_root, block.receipts_root);
        assert_eq!(decoded.proposer_public_key, block.proposer_public_key);
        assert_eq!(decoded.proposer_fee_recipient, block.proposer_fee_recipient);
        assert_eq!(decoded.extra_data, block.extra_data);
        assert_eq!(decoded.gas_used, block.gas_used);
        assert_eq!(decoded.base_fee_per_gas, block.base_fee_per_gas);
        assert_eq!(decoded.timestamp, block.timestamp);
        assert_eq!(decoded.transactions, block.transactions);
    }

    #[test]
    fn test_execution_result_fields() {
        let result = ExecutionResult {
            state_root: [2u8; 32],
            receipts_root: [3u8; 32],
            gas_used: 100,
            receipt_count: 5,
        };

        assert_eq!(result.state_root, [2u8; 32]);
        assert_eq!(result.receipts_root, [3u8; 32]);
        assert_eq!(result.gas_used, 100);
        assert_eq!(result.receipt_count, 5);
    }

    #[test]
    fn test_canonical_extra_data_roundtrip_with_raw_eth_and_full_dkg() {
        let original = CanonicalExtraDataV1 {
            raw_eth: Some(vec![0x11; 32]),
            full_dkg: Some(FullDkgV1 {
                epoch: 7,
                output: FullDkgOutputV1 {
                    dealers: vec![[0x22; 32], [0x23; 32]],
                    players: vec![[0x31; 32], [0x32; 32]],
                    public_polynomial: vec![0xaa, 0xbb, 0xcc],
                },
            }),
            reshare: Some(ReshareV1 {
                target_epoch: 9,
                players: vec![[0x41; 32], [0x42; 32]],
            }),
        };

        let encoded = encode_canonical_extra_data(&original).expect("encode");
        let decoded = decode_extra_data(&encoded, ExtraDataDecodeMode::Strict).expect("decode");
        assert_eq!(decoded, original);

        let projected = project_raw_eth_extra_data(&encoded);
        assert_eq!(projected, vec![0x11; 32]);
    }

    #[test]
    fn test_legacy_extra_data_decode_and_projection() {
        let legacy = vec![0x55; 32];
        let decoded = decode_extra_data(&legacy, ExtraDataDecodeMode::Legacy).expect("legacy");
        assert_eq!(decoded.raw_eth, Some(legacy.clone()));
        assert_eq!(decoded.full_dkg, None);
        assert_eq!(project_raw_eth_extra_data(&legacy), legacy);
    }

    #[test]
    fn test_strict_mode_rejects_legacy_bytes() {
        let legacy = vec![0x11; 32];
        assert!(decode_extra_data(&legacy, ExtraDataDecodeMode::Strict).is_err());
    }

    #[test]
    fn test_unknown_section_rejected() {
        let mut encoded = vec![];
        encoded.extend_from_slice(b"WDX1");
        encoded.push(1); // version
        encoded.push(1); // section count
        encoded.push(9); // unknown section id
        encoded.extend_from_slice(&1u32.to_le_bytes());
        encoded.push(0xaa);

        assert!(decode_extra_data(&encoded, ExtraDataDecodeMode::Legacy).is_err());
    }

    #[test]
    fn test_section_order_rejected_when_raw_eth_after_full_dkg() {
        let canonical = encode_canonical_extra_data(&CanonicalExtraDataV1 {
            raw_eth: Some(vec![0x11; 32]),
            full_dkg: Some(FullDkgV1 {
                epoch: 2,
                output: FullDkgOutputV1 {
                    dealers: vec![[0x21; 32]],
                    players: vec![[0x31; 32]],
                    public_polynomial: vec![0xaa, 0xbb],
                },
            }),
            reshare: None,
        })
        .expect("canonical envelope should encode");

        let mut cursor = &canonical[6..];
        let section1_id = cursor[0];
        let section1_len = u32::from_le_bytes(cursor[1..5].try_into().expect("len bytes")) as usize;
        let section1_payload = cursor[5..5 + section1_len].to_vec();
        cursor = &cursor[5 + section1_len..];

        let section2_id = cursor[0];
        let section2_len = u32::from_le_bytes(cursor[1..5].try_into().expect("len bytes")) as usize;
        let section2_payload = cursor[5..5 + section2_len].to_vec();
        assert_eq!(section1_id, 1, "first canonical section should be raw_eth");
        assert_eq!(
            section2_id, 2,
            "second canonical section should be full_dkg"
        );

        let mut reordered = Vec::new();
        reordered.extend_from_slice(b"WDX1");
        reordered.push(1);
        reordered.push(2);

        reordered.push(section2_id);
        reordered.extend_from_slice(&(section2_payload.len() as u32).to_le_bytes());
        reordered.extend_from_slice(&section2_payload);

        reordered.push(section1_id);
        reordered.extend_from_slice(&(section1_payload.len() as u32).to_le_bytes());
        reordered.extend_from_slice(&section1_payload);

        let err = decode_extra_data(&reordered, ExtraDataDecodeMode::Strict)
            .expect_err("raw_eth after full_dkg must be rejected");
        assert!(matches!(
            err,
            ExtraDataError::InvalidSectionOrder { section } if section == 1
        ));
    }

    #[test]
    fn test_section_order_rejected_when_reshare_before_full_dkg() {
        let canonical = encode_canonical_extra_data(&CanonicalExtraDataV1 {
            raw_eth: Some(vec![0x11; 32]),
            full_dkg: Some(FullDkgV1 {
                epoch: 2,
                output: FullDkgOutputV1 {
                    dealers: vec![[0x21; 32]],
                    players: vec![[0x31; 32]],
                    public_polynomial: vec![0xaa, 0xbb],
                },
            }),
            reshare: Some(ReshareV1 {
                target_epoch: 3,
                players: vec![[0x41; 32]],
            }),
        })
        .expect("canonical envelope should encode");

        let mut cursor = &canonical[6..];
        let raw_id = cursor[0];
        let raw_len = u32::from_le_bytes(cursor[1..5].try_into().expect("len bytes")) as usize;
        let raw_payload = cursor[5..5 + raw_len].to_vec();
        cursor = &cursor[5 + raw_len..];

        let full_dkg_id = cursor[0];
        let full_dkg_len = u32::from_le_bytes(cursor[1..5].try_into().expect("len bytes")) as usize;
        let full_dkg_payload = cursor[5..5 + full_dkg_len].to_vec();
        cursor = &cursor[5 + full_dkg_len..];

        let reshare_id = cursor[0];
        let reshare_len = u32::from_le_bytes(cursor[1..5].try_into().expect("len bytes")) as usize;
        let reshare_payload = cursor[5..5 + reshare_len].to_vec();

        let mut reordered = Vec::new();
        reordered.extend_from_slice(b"WDX1");
        reordered.push(1);
        reordered.push(3);

        reordered.push(raw_id);
        reordered.extend_from_slice(&(raw_payload.len() as u32).to_le_bytes());
        reordered.extend_from_slice(&raw_payload);

        reordered.push(reshare_id);
        reordered.extend_from_slice(&(reshare_payload.len() as u32).to_le_bytes());
        reordered.extend_from_slice(&reshare_payload);

        reordered.push(full_dkg_id);
        reordered.extend_from_slice(&(full_dkg_payload.len() as u32).to_le_bytes());
        reordered.extend_from_slice(&full_dkg_payload);

        let err = decode_extra_data(&reordered, ExtraDataDecodeMode::Strict)
            .expect_err("reshare before full_dkg must be rejected");
        assert!(matches!(
            err,
            ExtraDataError::InvalidSectionOrder { section } if section == 3
        ));
    }
}
