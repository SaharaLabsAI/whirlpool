pub fn raw_tx_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}
