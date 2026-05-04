use crate::core::events::*;
use crate::instr::inner_common::*;
use solana_sdk::pubkey::Pubkey;

//
// ## 解析器插件系统
//
// 支持两种可插拔的解析器实现：
//
// ### 1. Borsh 反序列化解析器（默认，推荐）
// - **启用**: `cargo build --features parse-borsh` （默认）
// - 特点：类型安全、代码简洁、易于维护
//
// ### 2. 零拷贝解析器（高性能）
// - **启用**: `cargo build --features parse-zero-copy --no-default-features`
// - 特点：最高性能、零内存分配、直接读取内存


pub mod discriminators {
    pub const TRADED: [u8; 16] =
        [225, 202, 73, 175, 147, 43, 160, 150, 155, 167, 108, 32, 122, 76, 173, 64];
    pub const LIQUIDITY_INCREASED: [u8; 16] =
        [30, 7, 144, 181, 102, 254, 155, 161, 155, 167, 108, 32, 122, 76, 173, 64];
    pub const LIQUIDITY_DECREASED: [u8; 16] =
        [166, 1, 36, 71, 112, 202, 181, 171, 155, 167, 108, 32, 122, 76, 173, 64];
    pub const POOL_INITIALIZED: [u8; 16] =
        [100, 118, 173, 87, 12, 198, 254, 229, 155, 167, 108, 32, 122, 76, 173, 64];
}

/// 主入口：根据 discriminator 解析事件
#[inline]
pub fn parse(disc: &[u8; 16], data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    match disc {
        &discriminators::TRADED => parse_swap(data, metadata),
        &discriminators::LIQUIDITY_INCREASED => parse_liquidity_increased(data, metadata),
        &discriminators::LIQUIDITY_DECREASED => parse_liquidity_decreased(data, metadata),
        _ => None,
    }
}

// ============================================================================
// Swap Event (Traded)
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

/// Borsh 解析器
#[cfg(feature = "parse-borsh")]
#[inline(always)]
fn parse_swap_borsh(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    // 数据结构：whirlpool(32) + input_amount(8) + output_amount(8) + a_to_b(1) = 49 bytes
    const SWAP_EVENT_SIZE: usize = 32 + 8 + 8 + 1;
    if data.len() < SWAP_EVENT_SIZE {
        return None;
    }

    let mut event =
        borsh::from_slice::<OrcaWhirlpoolSwapEvent>(&data[..SWAP_EVENT_SIZE]).ok()?;
    event.metadata = metadata;
    Some(DexEvent::OrcaWhirlpoolSwap(event))
}

/// 零拷贝解析器
#[cfg(feature = "parse-zero-copy")]
#[inline(always)]
fn parse_swap_zero_copy(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    unsafe {
        if !check_length(data, 32 + 8 + 8 + 1) {
            return None;
        }
        let whirlpool = read_pubkey_unchecked(data, 0);
        let input_amount = read_u64_unchecked(data, 32);
        let output_amount = read_u64_unchecked(data, 40);
        let a_to_b = read_bool_unchecked(data, 48);
        Some(DexEvent::OrcaWhirlpoolSwap(OrcaWhirlpoolSwapEvent {
            metadata,
            whirlpool,
            input_amount,
            output_amount,
            a_to_b,
            pre_sqrt_price: 0,
            post_sqrt_price: 0,
            input_transfer_fee: 0,
            output_transfer_fee: 0,
            lp_fee: 0,
            protocol_fee: 0,
        }))
    }
}

// ============================================================================
// LiquidityIncreased Event
// ============================================================================

/// 解析 LiquidityIncreased 事件（统一入口）
#[inline(always)]
fn parse_liquidity_increased(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    #[cfg(feature = "parse-borsh")]
    {
        parse_liquidity_increased_borsh(data, metadata)
    }

    #[cfg(feature = "parse-zero-copy")]
    {
        parse_liquidity_increased_zero_copy(data, metadata)
    }
}

/// Borsh 解析器
#[cfg(feature = "parse-borsh")]
#[inline(always)]
fn parse_liquidity_increased_borsh(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    // 数据结构：whirlpool(32) + liquidity(16) + token_a_amount(8) + token_b_amount(8) = 64 bytes
    const LIQUIDITY_EVENT_SIZE: usize = 32 + 16 + 8 + 8;
    if data.len() < LIQUIDITY_EVENT_SIZE {
        return None;
    }

    let mut event = borsh::from_slice::<OrcaWhirlpoolLiquidityIncreasedEvent>(
        &data[..LIQUIDITY_EVENT_SIZE],
    )
    .ok()?;
    event.metadata = metadata;
    Some(DexEvent::OrcaWhirlpoolLiquidityIncreased(event))
}

/// 零拷贝解析器
#[cfg(feature = "parse-zero-copy")]
#[inline(always)]
fn parse_liquidity_increased_zero_copy(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    unsafe {
        if !check_length(data, 32 + 16 + 8 + 8) {
            return None;
        }
        let whirlpool = read_pubkey_unchecked(data, 0);
        let liquidity = read_u128_unchecked(data, 32);
        let token_a_amount = read_u64_unchecked(data, 48);
        let token_b_amount = read_u64_unchecked(data, 56);
        Some(DexEvent::OrcaWhirlpoolLiquidityIncreased(OrcaWhirlpoolLiquidityIncreasedEvent {
            metadata,
            whirlpool,
            liquidity,
            token_a_amount,
            token_b_amount,
            position: Pubkey::default(),
            tick_lower_index: 0,
            tick_upper_index: 0,
            token_a_transfer_fee: 0,
            token_b_transfer_fee: 0,
        }))
    }
}

// ============================================================================
// LiquidityDecreased Event
// ============================================================================

/// 解析 LiquidityDecreased 事件（统一入口）
#[inline(always)]
fn parse_liquidity_decreased(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    #[cfg(feature = "parse-borsh")]
    {
        parse_liquidity_decreased_borsh(data, metadata)
    }

    #[cfg(feature = "parse-zero-copy")]
    {
        parse_liquidity_decreased_zero_copy(data, metadata)
    }
}

/// Borsh 解析器
#[cfg(feature = "parse-borsh")]
#[inline(always)]
fn parse_liquidity_decreased_borsh(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    // 数据结构：whirlpool(32) + liquidity(16) + token_a_amount(8) + token_b_amount(8) = 64 bytes
    const LIQUIDITY_EVENT_SIZE: usize = 32 + 16 + 8 + 8;
    if data.len() < LIQUIDITY_EVENT_SIZE {
        return None;
    }

    let mut event = borsh::from_slice::<OrcaWhirlpoolLiquidityDecreasedEvent>(
        &data[..LIQUIDITY_EVENT_SIZE],
    )
    .ok()?;
    event.metadata = metadata;
    Some(DexEvent::OrcaWhirlpoolLiquidityDecreased(event))
}

/// 零拷贝解析器
#[cfg(feature = "parse-zero-copy")]
#[inline(always)]
fn parse_liquidity_decreased_zero_copy(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    unsafe {
        if !check_length(data, 32 + 16 + 8 + 8) {
            return None;
        }
        let whirlpool = read_pubkey_unchecked(data, 0);
        let liquidity = read_u128_unchecked(data, 32);
        let token_a_amount = read_u64_unchecked(data, 48);
        let token_b_amount = read_u64_unchecked(data, 56);
        Some(DexEvent::OrcaWhirlpoolLiquidityDecreased(OrcaWhirlpoolLiquidityDecreasedEvent {
            metadata,
            whirlpool,
            liquidity,
            token_a_amount,
            token_b_amount,
            position: Pubkey::default(),
            tick_lower_index: 0,
            tick_upper_index: 0,
            token_a_transfer_fee: 0,
            token_b_transfer_fee: 0,
        }))
    }
}
