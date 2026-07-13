use serde::Serialize;

const KEYCHAIN_SERVICE: &str = "com.ohenry.screenpebble.claude-api";
const KEYCHAIN_ACCOUNT: &str = "anthropic-api-key";
const ERR_SEC_ITEM_NOT_FOUND: i32 = -25_300;
const MIN_API_KEY_BYTES: usize = 20;
const MAX_API_KEY_BYTES: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeCredentialError {
    InvalidKey,
    StoreUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCredentialStatus {
    pub api_key_configured: bool,
}

pub struct StoredSecret {
    bytes: Vec<u8>,
}

impl StoredSecret {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl Drop for StoredSecret {
    fn drop(&mut self) {
        self.bytes.fill(0);
    }
}

pub fn credential_status() -> Result<ClaudeCredentialStatus, ClaudeCredentialError> {
    Ok(ClaudeCredentialStatus {
        api_key_configured: load_api_key()?.is_some(),
    })
}

pub fn store_api_key(api_key: String) -> Result<ClaudeCredentialStatus, ClaudeCredentialError> {
    let mut bytes = api_key.into_bytes();
    let result = if valid_api_key(&bytes) {
        store_secret(&bytes)
    } else {
        Err(ClaudeCredentialError::InvalidKey)
    };
    bytes.fill(0);
    result?;
    Ok(ClaudeCredentialStatus {
        api_key_configured: true,
    })
}

pub fn delete_api_key() -> Result<ClaudeCredentialStatus, ClaudeCredentialError> {
    delete_secret()?;
    Ok(ClaudeCredentialStatus {
        api_key_configured: false,
    })
}

#[cfg(target_os = "macos")]
pub fn load_api_key() -> Result<Option<StoredSecret>, ClaudeCredentialError> {
    use security_framework::passwords::get_generic_password;

    match get_generic_password(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT) {
        Ok(bytes) if valid_api_key(&bytes) => Ok(Some(StoredSecret { bytes })),
        Ok(mut bytes) => {
            bytes.fill(0);
            Err(ClaudeCredentialError::StoreUnavailable)
        }
        Err(error) if error.code() == ERR_SEC_ITEM_NOT_FOUND => Ok(None),
        Err(_) => Err(ClaudeCredentialError::StoreUnavailable),
    }
}

#[cfg(not(target_os = "macos"))]
pub fn load_api_key() -> Result<Option<StoredSecret>, ClaudeCredentialError> {
    Err(ClaudeCredentialError::StoreUnavailable)
}

#[cfg(target_os = "macos")]
fn store_secret(bytes: &[u8]) -> Result<(), ClaudeCredentialError> {
    security_framework::passwords::set_generic_password(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT, bytes)
        .map_err(|_| ClaudeCredentialError::StoreUnavailable)
}

#[cfg(not(target_os = "macos"))]
fn store_secret(_bytes: &[u8]) -> Result<(), ClaudeCredentialError> {
    Err(ClaudeCredentialError::StoreUnavailable)
}

#[cfg(target_os = "macos")]
fn delete_secret() -> Result<(), ClaudeCredentialError> {
    use security_framework::passwords::delete_generic_password;

    match delete_generic_password(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT) {
        Ok(()) => Ok(()),
        Err(error) if error.code() == ERR_SEC_ITEM_NOT_FOUND => Ok(()),
        Err(_) => Err(ClaudeCredentialError::StoreUnavailable),
    }
}

#[cfg(not(target_os = "macos"))]
fn delete_secret() -> Result<(), ClaudeCredentialError> {
    Err(ClaudeCredentialError::StoreUnavailable)
}

fn valid_api_key(bytes: &[u8]) -> bool {
    (MIN_API_KEY_BYTES..=MAX_API_KEY_BYTES).contains(&bytes.len())
        && bytes.starts_with(b"sk-ant-")
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

#[cfg(test)]
mod tests {
    use super::valid_api_key;

    #[test]
    fn accepts_only_bounded_anthropic_api_keys() {
        let valid = format!("sk-ant-{}", "x".repeat(24));
        assert!(valid_api_key(valid.as_bytes()));
        assert!(!valid_api_key(b"sk-other-api03-valid_key-123456"));
        assert!(!valid_api_key(b"sk-ant-short"));
        assert!(!valid_api_key(b"sk-ant-api03-key with space"));
        assert!(!valid_api_key(&vec![b'a'; 513]));
    }
}
