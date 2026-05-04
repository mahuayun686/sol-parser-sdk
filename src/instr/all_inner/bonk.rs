use crate::core::events::*;
use crate::instr::inner_common::*;
use solana_sdk::pubkey::Pubkey;

// Bonk (Raydium Launchpad) Inner Instruction 解析器
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
    pub const POOL_CREATE: [u8; 16] =
        [100, 50, 200, 150, 75, 120, 90, 30, 155, 167, 108, 32, 122, 76, 173, 64];
    pub const TRADE: [u8; 16] =
        [80, 120, 100, 200, 150, 75, 60, 40, 155, 167, 108, 32, 122, 76, 173, 64];
    pub const MIGRATE: [u8; 16] =
        [90, 130, 110, 210, 160, 85, 70, 50, 155, 167, 108, 32, 122, 76, 173, 64];
}

/// 主入口：根据 discriminator 解析事件
#[inline]
pub fn parse(disc: &[u8; 16], data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    match disc {
        &discriminators::TRADE => parse_trade(data, metadata),
        _ => None,
    }
}

// ============================================================================
// Trade Event
// ============================================================================

/// 解析 Trade 事件（统一入口）
#[inline(always)]
fn parse_trade(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    #[cfg(feature = "parse-borsh")]
    {
        parse_trade_borsh(data, metadata)
    }

    #[cfg(feature = "parse-zero-copy")]
    {
        parse_trade_zero_copy(data, metadata)
    }
}

/// Borsh 解析器
#[cfg(feature = "parse-borsh")]
#[inline(always)]
fn parse_trade_borsh(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    // 数据结构：pool_state(32) + user(32) + amount_in(8) + amount_out(8) + is_buy(1) = 81 bytes
    const TRADE_EVENT_SIZE: usize = 32 + 32 + 8 + 8 + 1;
    if data.len() < TRADE_EVENT_SIZE {
        return None;
    }

    let mut event = borsh::from_slice::<BonkTradeEvent>(&data[..TRADE_EVENT_SIZE]).ok()?;
    event.metadata = metadata;
    event.trade_direction =
        if event.is_buy { TradeDirection::Buy } else { TradeDirection::Sell };
    event.exact_in = true;
    Some(DexEvent::BonkTrade(event))
}

/// 零拷贝解析器
#[cfg(feature = "parse-zero-copy")]
#[inline(always)]
fn parse_trade_zero_copy(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    unsafe {
        if !check_length(data, 32 + 32 + 8 + 8 + 1) {
            return None;
        }
        let pool_state = read_pubkey_unchecked(data, 0);
        let user = read_pubkey_unchecked(data, 32);
        let amount_in = read_u64_unchecked(data, 64);
        let amount_out = read_u64_unchecked(data, 72);
        let is_buy = read_bool_unchecked(data, 80);
        Some(DexEvent::BonkTrade(BonkTradeEvent {
            metadata,
            pool_state,
            user,
            amount_in,
            amount_out,
            is_buy,
            trade_direction: if is_buy { TradeDirection::Buy } else { TradeDirection::Sell },
            exact_in: true,
        }))
    }
}
