use crate::core::events::*;
use crate::instr::inner_common::*;
use solana_sdk::pubkey::Pubkey;

pub mod discriminators {
    pub const SWAP_BASE_IN: [u8; 16] =
        [143, 190, 90, 218, 196, 30, 51, 222, 155, 167, 108, 32, 122, 76, 173, 64];
    pub const SWAP_BASE_OUT: [u8; 16] =
        [55, 217, 98, 86, 163, 74, 180, 173, 155, 167, 108, 32, 122, 76, 173, 64];
    pub const CREATE_POOL: [u8; 16] =
        [233, 146, 209, 142, 207, 104, 64, 188, 155, 167, 108, 32, 122, 76, 173, 64];
    pub const DEPOSIT: [u8; 16] =
        [242, 35, 198, 137, 82, 225, 242, 182, 155, 167, 108, 32, 122, 76, 173, 64];
    pub const WITHDRAW: [u8; 16] =
        [183, 18, 70, 156, 148, 109, 161, 34, 155, 167, 108, 32, 122, 76, 173, 64];
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
    #[cfg(feature = "parse-borsh")]
    {
        parse_swap_borsh(data, metadata)
    }

    #[cfg(feature = "parse-zero-copy")]
    {
        parse_swap_zero_copy(data, metadata)
    }
}

/// Borsh 反序列化解析器 - Swap 事件
#[cfg(feature = "parse-borsh")]
#[inline(always)]
fn parse_swap_borsh(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    // 数据结构:
    // pool_id: Pubkey (32 bytes)
    // input_amount: u64 (8 bytes)
    // output_amount: u64 (8 bytes)
    // Total: 48 bytes
    const EVENT_SIZE: usize = 32 + 8 + 8;

    if data.len() < EVENT_SIZE {
        return None;
    }

    let event = borsh::from_slice::<RaydiumCpmmSwapEvent>(&data[..EVENT_SIZE]).ok()?;

    Some(DexEvent::RaydiumCpmmSwap(RaydiumCpmmSwapEvent { metadata, ..event }))
}

/// 零拷贝解析器 - Swap 事件
#[cfg(feature = "parse-zero-copy")]
#[inline(always)]
fn parse_swap_zero_copy(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    unsafe {
        if !check_length(data, 32 + 8 + 8) {
            return None;
        }
        let pool = read_pubkey_unchecked(data, 0);
        let input_amount = read_u64_unchecked(data, 32);
        let output_amount = read_u64_unchecked(data, 40);
        Some(DexEvent::RaydiumCpmmSwap(RaydiumCpmmSwapEvent {
            metadata,
            pool_id: pool,
            input_amount,
            output_amount,
            input_vault_before: 0,
            output_vault_before: 0,
            input_transfer_fee: 0,
            output_transfer_fee: 0,
            base_input: true,
        }))
    }
}

// ============================================================================
// Deposit 事件解析器
// ============================================================================

/// 解析 Deposit 事件（统一入口）
#[inline(always)]
fn parse_deposit(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    #[cfg(feature = "parse-borsh")]
    {
        parse_deposit_borsh(data, metadata)
    }

    #[cfg(feature = "parse-zero-copy")]
    {
        parse_deposit_zero_copy(data, metadata)
    }
}

/// Borsh 反序列化解析器 - Deposit 事件
#[cfg(feature = "parse-borsh")]
#[inline(always)]
fn parse_deposit_borsh(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    // 数据结构:
    // pool: Pubkey (32 bytes)
    // token0_amount: u64 (8 bytes)
    // token1_amount: u64 (8 bytes)
    // lp_token_amount: u64 (8 bytes)
    // Total: 56 bytes
    const EVENT_SIZE: usize = 32 + 8 + 8 + 8;

    if data.len() < EVENT_SIZE {
        return None;
    }

    let event = borsh::from_slice::<RaydiumCpmmDepositEvent>(&data[..EVENT_SIZE]).ok()?;

    Some(DexEvent::RaydiumCpmmDeposit(RaydiumCpmmDepositEvent { metadata, ..event }))
}

/// 零拷贝解析器 - Deposit 事件
#[cfg(feature = "parse-zero-copy")]
#[inline(always)]
fn parse_deposit_zero_copy(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    unsafe {
        if !check_length(data, 32 + 8 + 8 + 8) {
            return None;
        }
        let pool = read_pubkey_unchecked(data, 0);
        let token0_amount = read_u64_unchecked(data, 32);
        let token1_amount = read_u64_unchecked(data, 40);
        let lp_token_amount = read_u64_unchecked(data, 48);
        Some(DexEvent::RaydiumCpmmDeposit(RaydiumCpmmDepositEvent {
            metadata,
            pool,
            lp_token_amount,
            token0_amount,
            token1_amount,
            user: Pubkey::default(),
        }))
    }
}

// ============================================================================
// Withdraw 事件解析器
// ============================================================================

/// 解析 Withdraw 事件（统一入口）
#[inline(always)]
fn parse_withdraw(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    #[cfg(feature = "parse-borsh")]
    {
        parse_withdraw_borsh(data, metadata)
    }

    #[cfg(feature = "parse-zero-copy")]
    {
        parse_withdraw_zero_copy(data, metadata)
    }
}

/// Borsh 反序列化解析器 - Withdraw 事件
#[cfg(feature = "parse-borsh")]
#[inline(always)]
fn parse_withdraw_borsh(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    // 数据结构:
    // pool: Pubkey (32 bytes)
    // lp_token_amount: u64 (8 bytes)
    // token0_amount: u64 (8 bytes)
    // token1_amount: u64 (8 bytes)
    // Total: 56 bytes
    const EVENT_SIZE: usize = 32 + 8 + 8 + 8;

    if data.len() < EVENT_SIZE {
        return None;
    }

    let event = borsh::from_slice::<RaydiumCpmmWithdrawEvent>(&data[..EVENT_SIZE]).ok()?;

    Some(DexEvent::RaydiumCpmmWithdraw(RaydiumCpmmWithdrawEvent { metadata, ..event }))
}

/// 零拷贝解析器 - Withdraw 事件
#[cfg(feature = "parse-zero-copy")]
#[inline(always)]
fn parse_withdraw_zero_copy(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    unsafe {
        if !check_length(data, 32 + 8 + 8 + 8) {
            return None;
        }
        let pool = read_pubkey_unchecked(data, 0);
        let lp_token_amount = read_u64_unchecked(data, 32);
        let token0_amount = read_u64_unchecked(data, 40);
        let token1_amount = read_u64_unchecked(data, 48);
        Some(DexEvent::RaydiumCpmmWithdraw(RaydiumCpmmWithdrawEvent {
            metadata,
            pool,
            lp_token_amount,
            token0_amount,
            token1_amount,
            user: Pubkey::default(),
        }))
    }
}
