pub mod nonce;
pub mod program_ids;
pub mod pumpswap;
pub mod rpc_wallet;
pub mod token;
pub mod utils;
use crate::core::events::EventMetadata;
use crate::grpc::EventTypeFilter;
use crate::DexEvent;
pub use nonce::parse_nonce_account;
use program_ids::*;
pub use pumpswap::{
    parse_global_config as parse_pumpswap_global_config, parse_pool as parse_pumpswap_pool,
};
pub use rpc_wallet::rpc_resolve_user_wallet_pubkey;
pub use token::parse_token_account;
pub use token::AccountData;
pub use utils::*;

pub fn parse_account_unified(
    account: &AccountData,
    metadata: EventMetadata,
    event_type_filter: Option<&EventTypeFilter>,
) -> Option<DexEvent> {
    if account.data.is_empty() {
        return None;
    }

    // Early filtering based on event type filter
    if let Some(filter) = event_type_filter {
        if let Some(ref include_only) = filter.include_only {
            // Check if any of the account event types are in the include list
            let should_parse = include_only.iter().any(|t| {
                use crate::grpc::EventType;
                matches!(
                    t,
                    EventType::TokenAccount
                        | EventType::NonceAccount
                        | EventType::AccountPumpFunGlobal
                        | EventType::AccountPumpSwapGlobalConfig
                        | EventType::AccountPumpSwapPool
                )
            });
            if !should_parse {
                return None;
            }
        }
    }

    if account.owner == PUMPSWAP_PROGRAM_ID {
        if let Some(filter) = event_type_filter {
            if filter.should_include(crate::grpc::EventType::AccountPumpSwapGlobalConfig)
                || filter.should_include(crate::grpc::EventType::AccountPumpSwapPool)
            {
                let event = parse_pumpswap_account(account, metadata.clone());
                if event.is_some() {
                    return event;
                }
            }
        }
    }
    if account.owner == crate::grpc::program_ids::PUMPFUN_PROGRAM {
        if let Some(filter) = event_type_filter {
            if filter.should_include(crate::grpc::EventType::AccountPumpFunGlobal) {
                let event = parse_pumpfun_account(account, metadata.clone());
                if event.is_some() {
                    return event;
                }
            }
        }
    }
    if nonce::is_nonce_account(&account.data) {
        // Check filter for NonceAccount specifically
        if let Some(filter) = event_type_filter {
            if !filter.should_include(crate::grpc::EventType::NonceAccount) {
                return None;
            }
        }
        return parse_nonce_account(account, metadata);
    }
    // Parse token account (includes both TokenAccount and TokenInfo)
    if let Some(filter) = event_type_filter {
        let includes_token = filter.should_include(crate::grpc::EventType::TokenAccount);
        if !includes_token {
            return None;
        }
    }
    return parse_token_account(account, metadata);
}

fn parse_pumpswap_account(account: &AccountData, metadata: EventMetadata) -> Option<DexEvent> {
    // 检查 discriminator 以确定账户类型
    if pumpswap::is_global_config_account(&account.data) {
        return pumpswap::parse_global_config(account, metadata);
    }
    if pumpswap::is_pool_account(&account.data) {
        return pumpswap::parse_pool(account, metadata);
    }
    None
}

fn parse_pumpfun_account(account: &AccountData, metadata: EventMetadata) -> Option<DexEvent> {
    use crate::core::events::{PumpFunGlobal, PumpFunGlobalAccountEvent};

    const GLOBAL_DISCRIMINATOR: &[u8; 8] = &[167, 232, 232, 177, 200, 108, 114, 127];
    if !has_discriminator(&account.data, GLOBAL_DISCRIMINATOR) {
        return None;
    }

    let data = &account.data[8..];
    let mut offset = 0usize;
    let initialized = read_u8(data, offset)? != 0;
    offset += 1;
    let authority = read_pubkey(data, offset)?;
    offset += 32;
    let fee_recipient = read_pubkey(data, offset)?;
    offset += 32;
    let initial_virtual_token_reserves = read_u64_le(data, offset)?;
    offset += 8;
    let initial_virtual_sol_reserves = read_u64_le(data, offset)?;
    offset += 8;
    let initial_real_token_reserves = read_u64_le(data, offset)?;
    offset += 8;
    let token_total_supply = read_u64_le(data, offset)?;
    offset += 8;
    let fee_basis_points = read_u64_le(data, offset)?;
    offset += 8;
    let withdraw_authority = read_pubkey(data, offset)?;
    offset += 32;
    let enable_migrate = read_u8(data, offset)? != 0;
    offset += 1;
    let pool_migration_fee = read_u64_le(data, offset)?;
    offset += 8;
    let creator_fee_basis_points = read_u64_le(data, offset)?;
    offset += 8;
    let mut fee_recipients = [solana_sdk::pubkey::Pubkey::default(); 8];
    for i in 0..8 {
        fee_recipients[i] = read_pubkey(data, offset)?;
        offset += 32;
    }
    let set_creator_authority = read_pubkey(data, offset)?;
    offset += 32;
    let admin_set_creator_authority = read_pubkey(data, offset)?;
    offset += 32;
    let create_v2_enabled = read_u8(data, offset)? != 0;
    offset += 1;
    let whitelist_pda = read_pubkey(data, offset)?;
    offset += 32;
    let reserved_fee_recipient = read_pubkey(data, offset)?;
    offset += 32;
    let mayhem_mode_enabled = read_u8(data, offset)? != 0;
    offset += 1;
    let mut reserved_fee_recipients = [solana_sdk::pubkey::Pubkey::default(); 7];
    for i in 0..7 {
        reserved_fee_recipients[i] = read_pubkey(data, offset)?;
        offset += 32;
    }
    let _is_cashback_enabled = read_u8(data, offset)? != 0;
    offset += 1;
    let _buyback_fee_recipients = {
        let mut keys = [solana_sdk::pubkey::Pubkey::default(); 8];
        for i in 0..8 {
            keys[i] = read_pubkey(data, offset)?;
            offset += 32;
        }
        keys
    };

    let global = PumpFunGlobal {
        initialized,
        authority,
        fee_recipient,
        initial_virtual_token_reserves,
        initial_virtual_sol_reserves,
        initial_real_token_reserves,
        token_total_supply,
        fee_basis_points,
        withdraw_authority,
        enable_migrate,
        pool_migration_fee,
        creator_fee_basis_points,
        fee_recipients,
        set_creator_authority,
        admin_set_creator_authority,
        create_v2_enabled,
        whitelist_pda,
        reserved_fee_recipient,
        mayhem_mode_enabled,
        reserved_fee_recipients,
    };

    Some(DexEvent::PumpFunGlobalAccount(PumpFunGlobalAccountEvent {
        metadata,
        pubkey: account.pubkey,
        global,
    }))
}
