use crate::core::events::*;
use crate::instr::inner_common::*;
use solana_sdk::pubkey::Pubkey;

pub mod discriminators {
    pub const SWAP_BASE_IN: [u8; 16] =
        [0, 0, 0, 0, 0, 0, 0, 9, 155, 167, 108, 32, 122, 76, 173, 64];
    pub const SWAP_BASE_OUT: [u8; 16] =
        [0, 0, 0, 0, 0, 0, 0, 11, 155, 167, 108, 32, 122, 76, 173, 64];
    pub const DEPOSIT: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 3, 155, 167, 108, 32, 122, 76, 173, 64];
    pub const WITHDRAW: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 4, 155, 167, 108, 32, 122, 76, 173, 64];
    pub const INITIALIZE2: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 1, 155, 167, 108, 32, 122, 76, 173, 64];
}

/// 主入口：根据 discriminator 解析事件
#[inline]
pub fn parse(disc: &[u8; 16], data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    match disc {
        &discriminators::SWAP_BASE_IN | &discriminators::SWAP_BASE_OUT => {
            parse_swap(data, metadata)
        }
        &discriminators::DEPOSIT => parse_deposit(data, metadata),
        &discriminators::WITHDRAW => parse_withdraw(data, metadata),
        _ => None,
    }
}

// ============================================================================
// Swap 事件解析器
// ============================================================================

/// 解析 Swap 事件（统一入口）
#[inline(always)]
fn parse_swap(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    #[cfg(all(feature = "parse-borsh", not(feature = "parse-zero-copy")))]
    {
        parse_swap_borsh(data, metadata)
    }

    #[cfg(feature = "parse-zero-copy")]
    {
        parse_swap_zero_copy(data, metadata)
    }
}

/// Borsh 反序列化解析器 - Swap 事件
#[cfg(all(feature = "parse-borsh", not(feature = "parse-zero-copy")))]
#[inline(always)]
fn parse_swap_borsh(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    // 数据结构:
    // amm: Pubkey (32 bytes)
    // amount_in: u64 (8 bytes)
    // amount_out: u64 (8 bytes)
    // Total: 48 bytes
    const EVENT_SIZE: usize = 32 + 8 + 8;

    if data.len() < EVENT_SIZE {
        return None;
    }

    let event = borsh::from_slice::<RaydiumAmmV4SwapEvent>(&data[..EVENT_SIZE]).ok()?;

    Some(DexEvent::RaydiumAmmV4Swap(RaydiumAmmV4SwapEvent { metadata, ..event }))
}

/// 零拷贝解析器 - Swap 事件
#[cfg(feature = "parse-zero-copy")]
#[inline(always)]
fn parse_swap_zero_copy(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    unsafe {
        if !check_length(data, 32 + 8 + 8) {
            return None;
        }
        let amm = read_pubkey_unchecked(data, 0);
        let amount_in = read_u64_unchecked(data, 32);
        let amount_out = read_u64_unchecked(data, 40);
        Some(DexEvent::RaydiumAmmV4Swap(RaydiumAmmV4SwapEvent {
            metadata,
            amm,
            amount_in,
            amount_out,
            minimum_amount_out: 0,
            max_amount_in: 0,
            token_program: Pubkey::default(),
            amm_authority: Pubkey::default(),
            amm_open_orders: Pubkey::default(),
            amm_target_orders: None,
            pool_coin_token_account: Pubkey::default(),
            pool_pc_token_account: Pubkey::default(),
            serum_program: Pubkey::default(),
            serum_market: Pubkey::default(),
            serum_bids: Pubkey::default(),
            serum_asks: Pubkey::default(),
            serum_event_queue: Pubkey::default(),
            serum_coin_vault_account: Pubkey::default(),
            serum_pc_vault_account: Pubkey::default(),
            serum_vault_signer: Pubkey::default(),
            user_source_token_account: Pubkey::default(),
            user_destination_token_account: Pubkey::default(),
            user_source_owner: Pubkey::default(),
        }))
    }
}

// ============================================================================
// Deposit 事件解析器
// ============================================================================

/// 解析 Deposit 事件（统一入口）
#[inline(always)]
fn parse_deposit(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    #[cfg(all(feature = "parse-borsh", not(feature = "parse-zero-copy")))]
    {
        parse_deposit_borsh(data, metadata)
    }

    #[cfg(feature = "parse-zero-copy")]
    {
        parse_deposit_zero_copy(data, metadata)
    }
}

/// Borsh 反序列化解析器 - Deposit 事件
#[cfg(all(feature = "parse-borsh", not(feature = "parse-zero-copy")))]
#[inline(always)]
fn parse_deposit_borsh(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    // 数据结构:
    // amm: Pubkey (32 bytes)
    // max_coin_amount: u64 (8 bytes)
    // max_pc_amount: u64 (8 bytes)
    // Total: 48 bytes
    const EVENT_SIZE: usize = 32 + 8 + 8;

    if data.len() < EVENT_SIZE {
        return None;
    }

    let event = borsh::from_slice::<RaydiumAmmV4DepositEvent>(&data[..EVENT_SIZE]).ok()?;

    Some(DexEvent::RaydiumAmmV4Deposit(RaydiumAmmV4DepositEvent { metadata, ..event }))
}

/// 零拷贝解析器 - Deposit 事件
#[cfg(feature = "parse-zero-copy")]
#[inline(always)]
fn parse_deposit_zero_copy(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    unsafe {
        if !check_length(data, 32 + 8 + 8) {
            return None;
        }
        let amm = read_pubkey_unchecked(data, 0);
        let max_coin_amount = read_u64_unchecked(data, 32);
        let max_pc_amount = read_u64_unchecked(data, 40);
        Some(DexEvent::RaydiumAmmV4Deposit(RaydiumAmmV4DepositEvent {
            metadata,
            amm,
            max_coin_amount,
            max_pc_amount,
            base_side: 0,
            token_program: Pubkey::default(),
            amm_authority: Pubkey::default(),
            amm_open_orders: Pubkey::default(),
            amm_target_orders: Pubkey::default(),
            lp_mint_address: Pubkey::default(),
            pool_coin_token_account: Pubkey::default(),
            pool_pc_token_account: Pubkey::default(),
            serum_market: Pubkey::default(),
            serum_event_queue: Pubkey::default(),
            user_coin_token_account: Pubkey::default(),
            user_pc_token_account: Pubkey::default(),
            user_lp_token_account: Pubkey::default(),
            user_owner: Pubkey::default(),
        }))
    }
}

// ============================================================================
// Withdraw 事件解析器
// ============================================================================

/// 解析 Withdraw 事件（统一入口）
#[inline(always)]
fn parse_withdraw(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    #[cfg(all(feature = "parse-borsh", not(feature = "parse-zero-copy")))]
    {
        parse_withdraw_borsh(data, metadata)
    }

    #[cfg(feature = "parse-zero-copy")]
    {
        parse_withdraw_zero_copy(data, metadata)
    }
}

/// Borsh 反序列化解析器 - Withdraw 事件
#[cfg(all(feature = "parse-borsh", not(feature = "parse-zero-copy")))]
#[inline(always)]
fn parse_withdraw_borsh(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    // 数据结构:
    // amm: Pubkey (32 bytes)
    // amount: u64 (8 bytes)
    // Total: 40 bytes
    const EVENT_SIZE: usize = 32 + 8;

    if data.len() < EVENT_SIZE {
        return None;
    }

    let event = borsh::from_slice::<RaydiumAmmV4WithdrawEvent>(&data[..EVENT_SIZE]).ok()?;

    Some(DexEvent::RaydiumAmmV4Withdraw(RaydiumAmmV4WithdrawEvent { metadata, ..event }))
}

/// 零拷贝解析器 - Withdraw 事件
#[cfg(feature = "parse-zero-copy")]
#[inline(always)]
fn parse_withdraw_zero_copy(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    unsafe {
        if !check_length(data, 32 + 8) {
            return None;
        }
        let amm = read_pubkey_unchecked(data, 0);
        let amount = read_u64_unchecked(data, 32);
        Some(DexEvent::RaydiumAmmV4Withdraw(RaydiumAmmV4WithdrawEvent {
            metadata,
            amm,
            amount,
            token_program: Pubkey::default(),
            amm_authority: Pubkey::default(),
            amm_open_orders: Pubkey::default(),
            amm_target_orders: Pubkey::default(),
            lp_mint_address: Pubkey::default(),
            pool_coin_token_account: Pubkey::default(),
            pool_pc_token_account: Pubkey::default(),
            pool_withdraw_queue: Pubkey::default(),
            pool_temp_lp_token_account: Pubkey::default(),
            serum_program: Pubkey::default(),
            serum_market: Pubkey::default(),
            serum_bids: Pubkey::default(),
            serum_asks: Pubkey::default(),
            serum_event_queue: Pubkey::default(),
            serum_coin_vault_account: Pubkey::default(),
            serum_pc_vault_account: Pubkey::default(),
            serum_vault_signer: Pubkey::default(),
            user_lp_token_account: Pubkey::default(),
            user_coin_token_account: Pubkey::default(),
            user_pc_token_account: Pubkey::default(),
            user_owner: Pubkey::default(),
        }))
    }
}
