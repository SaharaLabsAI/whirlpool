use super::*;

pub fn parse_bootstrap_peer(s: &str) -> Result<BootstrapPeer, String> {
    let (public_key_hex, addr) = s
        .split_once('@')
        .ok_or_else(|| "bootstrap peer must be formatted as <pubkey>@<socket_addr>".to_string())?;

    let public_key_bytes = from_hex(public_key_hex)
        .ok_or_else(|| format!("invalid bootstrap peer public key hex: {public_key_hex}"))?;
    let mut reader = public_key_bytes.as_slice();
    let public_key = ed25519::PublicKey::read_cfg(&mut reader, &())
        .map_err(|err| format!("invalid bootstrap peer public key: {err}"))?;
    if !reader.is_empty() {
        return Err("invalid bootstrap peer public key length".to_string());
    }

    let addr = addr
        .parse()
        .map(Ingress::Socket)
        .map_err(|err| format!("invalid bootstrap peer socket address: {err}"))?;

    Ok((public_key, addr))
}

pub(super) fn parse_socket_addr(
    field: &'static str,
    value: String,
) -> Result<SocketAddr, ConfigError> {
    value
        .parse()
        .map_err(|source| ConfigError::InvalidSocketAddr {
            field,
            value,
            source,
        })
}

pub(super) fn parse_validator_hex(value: &str) -> Result<ed25519::PublicKey, String> {
    let bytes =
        from_hex(value).ok_or_else(|| format!("invalid validator public key hex: {value}"))?;
    let mut reader = bytes.as_slice();
    let public_key = ed25519::PublicKey::read_cfg(&mut reader, &())
        .map_err(|err| format!("invalid validator public key: {err}"))?;
    if !reader.is_empty() {
        return Err("invalid validator public key length".to_string());
    }
    Ok(public_key)
}

pub(super) fn parse_validator_list(
    values: Vec<String>,
) -> Result<Vec<ed25519::PublicKey>, ConfigError> {
    if values.is_empty() {
        return Err(ConfigError::EmptyBootstrapValidators);
    }

    values
        .into_iter()
        .map(|value| {
            parse_validator_hex(&value)
                .map_err(|reason| ConfigError::InvalidValidator { value, reason })
        })
        .collect()
}

pub(super) fn load_toml_config(path: &Path) -> Result<TomlConfig, ConfigError> {
    let contents = std::fs::read_to_string(path).map_err(|source| ConfigError::ReadConfig {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str(&contents).map_err(|source| ConfigError::ParseToml {
        path: path.to_path_buf(),
        source,
    })
}
