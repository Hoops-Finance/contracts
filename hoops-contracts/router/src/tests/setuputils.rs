use soroban_sdk::Address;

use super::test_setup::phoenix_factory;

pub fn generate_pho_lp_init_info(
    token_a: Address,
    token_b: Address,
    manager: Address,
    admin: Address,
    fee_recipient: Address,
) -> phoenix_factory::LiquidityPoolInitInfo {
    let token_init_info = phoenix_factory::TokenInitInfo { token_a, token_b };

    let stake_init_info = phoenix_factory::StakeInitInfo {
        min_bond: 10,
        min_reward: 10,
        manager,
        max_complexity: 10u32,
    };

    phoenix_factory::LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: fee_recipient.clone(),
        max_allowed_slippage_bps: 5000,
        max_allowed_spread_bps: 500,
        default_slippage_bps: 2_500,
        swap_fee_bps: 0,
        max_referral_bps: 5000,
        token_init_info,
        stake_init_info,
    }
}