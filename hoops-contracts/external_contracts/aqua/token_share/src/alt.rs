#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, token, Address, Env, Symbol, // Ensure token::Interface is available if needed, but direct client usage is more likely
    BytesN, String, // Keep String/BytesN if they become part of an extended interface
    token::{StellarAssetClient as SorobanTokenAdminClient, TokenClient as SorobanTokenClient},
};
use aqua_utils::bump::bump_instance;
use aqua_utils::storage_errors::StorageError; // Assuming this is for panic_with_error!

#[derive(Clone)]
#[contracttype]
enum DataKey {
    Admin,
    UnderlyingToken, // Address of the actual token contract holding the shares
    TotalManagedShares,  // Total supply of shares managed by this contract
}

// Helper to get the underlying token address
fn get_underlying_token(e: &Env) -> Address {
    bump_instance(e);
    match e.storage().instance().get(&DataKey::UnderlyingToken) {
        Some(v) => v,
        None => panic_with_error!(e, StorageError::ValueNotInitialized), // Or a more specific error
    }
}

// Helper to get total managed shares
fn get_total_managed_shares_internal(e: &Env) -> u128 {
    bump_instance(e);
    e.storage()
        .instance()
        .get(&DataKey::TotalManagedShares)
        .unwrap_or(0)
}

// Helper to set total managed shares
fn put_total_managed_shares_internal(e: &Env, value: u128) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::TotalManagedShares, &value)
}

pub trait LPShareManagerTrait {
    /// Initializes the LP Share Manager contract.
    ///
    /// # Arguments
    /// * `admin` - The administrative address for this manager contract.
    /// * `underlying_token_address` - The address of the deployed standard token contract
    ///   that will represent and store the actual LP share balances.
    fn initialize(e: Env, admin: Address, underlying_token_address: Address);

    /// Mints new LP shares.
    /// This function will call the `mint` function on the underlying token contract.
    /// Requires authorization (e.g., from the admin of this manager contract, or the liquidity pool contract if designated).
    ///
    /// # Arguments
    /// * `to` - The recipient of the new LP shares.
    /// * `amount` - The amount of LP shares to mint (as i128, consistent with token interface).
    fn mint_shares(e: Env, to: Address, amount: i128);

    /// Burns LP shares.
    /// This function will call the `burn` function on the underlying token contract.
    /// The `from` address must have authorized this manager contract or the burn operation directly on the underlying token.
    ///
    /// # Arguments
    /// * `from` - The address from which LP shares will be burned.
    /// * `amount` - The amount of LP shares to burn (as u128, to match original `burn_shares`).
    ///              Consider standardizing to i128 if the underlying token expects i128 for burn.
    fn burn_shares(e: Env, from: Address, amount: u128);

    /// Gets the LP share balance of a user.
    /// Reads the balance from the underlying token contract.
    ///
    /// # Arguments
    /// * `user` - The address of the user.
    /// # Returns
    /// The LP share balance of the user as u128.
    fn get_user_balance_shares(e: Env, user: Address) -> u128;

    /// Gets the total supply of LP shares managed by this contract.
    ///
    /// # Returns
    /// The total supply of LP shares as u128.
    fn get_total_managed_shares(e: Env) -> u128;

    // Add other functions if the liquidity pool needs them, e.g.,
    // fn transfer_shares(...)
    // fn approve_shares(...)
}

#[contract]
pub struct TokenShareContract;

#[contractimpl]
impl LPShareManagerTrait for TokenShareContract {
    fn initialize(e: Env, admin: Address, underlying_token_address: Address) {
        if e.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&e, Symbol::short("already_init"));
        }
        e.storage().instance().set(&DataKey::Admin, &admin);
        e.storage()
            .instance()
            .set(&DataKey::UnderlyingToken, &underlying_token_address);
        e.storage()
            .instance()
            .set(&DataKey::TotalManagedShares, &0_u128);
        bump_instance(&e);
    }

    fn mint_shares(e: Env, to: Address, amount: i128) {
        // Authorization: Typically, the liquidity pool contract (or the admin of this share manager)
        // would be authorized to call this.
        // For now, let's assume the admin of this TokenShareContract must authorize.
        // In a real scenario, the LP contract might be set as a "minter" role or similar.
        let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap_or_else(|| panic_with_error!(e, StorageError::ValueNotInitialized));
        admin.require_auth(); // Or some other auth logic, e.g. e.invoker().require_auth_for_args((&to, amount).into_val(&e));


        if amount <= 0 {
            panic_with_error!(&e, Symbol::short("amount_neg"));
        }
        let amount_u128 = amount as u128; // Assuming positive amount

        let total_shares = get_total_managed_shares_internal(&e);
        put_total_managed_shares_internal(&e, total_shares + amount_u128);

        let underlying_token = get_underlying_token(&e);
        // This TokenShareContract needs to be an admin/minter on the `underlying_token`
        // or the `admin` of this contract should be.
        // For simplicity, assuming this contract's admin can mint on underlying.
        SorobanTokenAdminClient::new(&e, &underlying_token).mint(&to, &amount);
        bump_instance(&e);
    }

    fn burn_shares(e: Env, from: Address, amount: u128) {
        // Authorization: The `from` address (owner of shares) must authorize the burn
        // on the underlying token. This contract facilitates that.
        // The `from.require_auth()` here means the `from` address is calling this `burn_shares` method.
        // If the LP contract calls this on behalf of `from`, then the LP contract needs `from`'s auth.
        from.require_auth();


        if amount == 0 {
            panic_with_error!(&e, Symbol::short("amount_zero"));
        }
        let amount_i128 = amount as i128; // Assuming it fits

        let total_shares = get_total_managed_shares_internal(&e);
        if total_shares < amount {
            panic_with_error!(&e, Symbol::short("insuf_supply"));
        }
        put_total_managed_shares_internal(&e, total_shares - amount);

        let underlying_token = get_underlying_token(&e);
        // The `from` address is burning its own tokens from the underlying contract.
        SorobanTokenClient::new(&e, &underlying_token).burn(&from, &amount_i128);
        bump_instance(&e);
    }

    fn get_user_balance_shares(e: Env, user: Address) -> u128 {
        bump_instance(&e);
        let underlying_token = get_underlying_token(&e);
        SorobanTokenClient::new(&e, &underlying_token).balance(&user) as u128
    }

    fn get_total_managed_shares(e: Env) -> u128 {
        bump_instance(&e);
        get_total_managed_shares_internal(&e)
    }
}

// Any direct `impl token::Interface for TokenShareContract` should be removed
// if this contract is a *manager* and not the token itself.
// The liquidity pool will interact with this manager via LPShareManagerTrait,
// and this manager interacts with an actual token via SorobanTokenClient/AdminClient.
