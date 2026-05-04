use crate::core::events::*;
use crate::instr::inner_common::*;
use solana_sdk::pubkey::Pubkey;


pub mod discriminators {
    pub const SWAP: [u8; 16] =
        [81, 108, 227, 190, 205, 208, 10, 196, 155, 167, 108, 32, 122, 76, 173, 64];
    pub const ADD_LIQUIDITY: [u8; 16] =
        [31, 94, 125, 90, 227, 52, 61, 186, 155, 167, 108, 32, 122, 76, 173, 64];
    pub const REMOVE_LIQUIDITY: [u8; 16] =
        [116, 244, 97, 232, 103, 31, 152, 58, 155, 167, 108, 32, 122, 76, 173, 64];
    pub const POOL_CREATED: [u8; 16] =
        [202, 44, 41, 88, 104, 220, 157, 82, 155, 167, 108, 32, 122, 76, 173, 64];
}

#[inline]
pub fn parse(disc: &[u8; 16], data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    unsafe {
        match disc {
            &discriminators::SWAP => {
                if !check_length(data, 8 + 8) {
                    return None;
                }
                let in_amount = read_u64_unchecked(data, 0);
                let out_amount = read_u64_unchecked(data, 8);
                Some(DexEvent::MeteoraPoolsSwap(MeteoraPoolsSwapEvent {
                    metadata,
                    in_amount,
                    out_amount,
                    trade_fee: 0,
                    admin_fee: 0,
                    host_fee: 0,
                }))
            }
            &discriminators::ADD_LIQUIDITY => {
                if !check_length(data, 8 + 8 + 8) {
                    return None;
                }
                let lp_mint_amount = read_u64_unchecked(data, 0);
                let token_a_amount = read_u64_unchecked(data, 8);
                let token_b_amount = read_u64_unchecked(data, 16);
                Some(DexEvent::MeteoraPoolsAddLiquidity(MeteoraPoolsAddLiquidityEvent {
                    metadata,
                    lp_mint_amount,
                    token_a_amount,
                    token_b_amount,
                }))
            }
            &discriminators::REMOVE_LIQUIDITY => {
                if !check_length(data, 8 + 8 + 8) {
                    return None;
                }
                let lp_unmint_amount = read_u64_unchecked(data, 0);
                let token_a_out_amount = read_u64_unchecked(data, 8);
                let token_b_out_amount = read_u64_unchecked(data, 16);
                Some(DexEvent::MeteoraPoolsRemoveLiquidity(MeteoraPoolsRemoveLiquidityEvent {
                    metadata,
                    lp_unmint_amount,
                    token_a_out_amount,
                    token_b_out_amount,
                }))
            }
            _ => None,
        }
    }
}
