use anchor_lang::prelude::*;
use crate::constants::*;
use crate::error::*;

/// Validate URI length
pub fn validate_uri(uri: &str) -> Result<()> {
    if uri.len() > MAX_URI_LENGTH {
        return err!(ValidationError::StringTooLong);
    }
    Ok(())
}

/// Validate collection name
pub fn validate_collection_name(name: &str) -> Result<()> {
    if name.len() > MAX_COLLECTION_NAME_LENGTH {
        return err!(ValidationError::StringTooLong);
    }
    if name.is_empty() {
        return err!(ValidationError::InvalidCollectionName);
    }
    Ok(())
}

/// Validate collection symbol
pub fn validate_collection_symbol(symbol: &str) -> Result<()> {
    if symbol.len() > MAX_COLLECTION_SYMBOL_LENGTH {
        return err!(ValidationError::StringTooLong);
    }
    if symbol.is_empty() {
        return err!(ValidationError::InvalidCollectionSymbol);
    }
    Ok(())
}

/// Validate message data length
pub fn validate_message_data(data: &[u8]) -> Result<()> {
    if data.len() > MAX_MESSAGE_LENGTH {
        return err!(ValidationError::StringTooLong);
    }
    Ok(())
}
