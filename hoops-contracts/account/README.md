# Hoops Account Contract

The `hoops-account` contract acts as a user-specific smart account designed to streamline interactions with the Hoops Finance `Router`. It enables users to manage their liquidity provision and redemption activities within the Hoops ecosystem.

## Core Functionality

The contract provides the following key features:

1.  **Lifecycle Management**:
    *   `initialize(owner: Address, router: Address)`: Sets the owner of the account and the address of the Hoops `Router` contract. This can only be called once.
    *   `upgrade(wasm: BytesN<32>)`: Allows the owner to upgrade the contract's WASM code.

2.  **Token Passthrough**:
    *   `transfer(token: Address, to: Address, amount: i128)`: A generic function allowing the owner to transfer any token held by this account to a specified address.

3.  **Liquidity Operations**:
    *   `deposit_usdc(usdc: Address, amount: i128, lp_plans: Vec<LpPlan>, deadline: u32)`:
        *   The owner first transfers USDC to this `Account` contract.
        *   This function then authorizes the `Account` contract to spend the owner's USDC.
        *   It approves the Hoops `Router` to pull the USDC from this `Account`.
        *   Finally, it calls `provide_liquidity` on the `Router` with the specified `amount` and `lp_plans` (detailing how the USDC should be allocated across different liquidity pools).
    *   `redeem(lp_token: Address, lp_amount: i128, usdc: Address, deadline: u32)`:
        *   The owner first approves this `Account` contract to spend their LP tokens.
        *   This function then approves the Hoops `Router` to pull the LP tokens from this `Account`.
        *   It calls `redeem_liquidity` on the `Router`.
        *   The USDC received from the `Router` is then automatically transferred (swept) to the owner's address.

4.  **View Functions**:
    *   `owner() -> Address`: Returns the address of the account owner.
    *   `router() -> Address`: Returns the address of the Hoops `Router` contract.

## Error Handling

The contract defines `AccountError` with the following variants:

*   `AlreadyInitialized = 1`: Raised if `initialize` is called more than once.
*   `NotAuthorized = 2`: Raised if a function requiring owner authorization is called by a different address.

## Events

The contract emits `TokenEvent { token: Address, amount: i128 }` for:
*   `("acct", "xfer")`: On successful `transfer`.
*   `("acct", "dep")`: On successful `deposit_usdc`.
*   `("acct", "wd")`: On successful `redeem` (logs the USDC amount withdrawn to the owner).

## Dependencies

*   `soroban-sdk`
*   `hoops-common`: For common types or utilities (though not explicitly used in the provided `lib.rs` for specific types beyond what `soroban-sdk` offers for this contract's logic).
*   `hoops-router`: For the `RouterClient` and `LpPlan` type, enabling interaction with the router contract. The router's WASM is imported directly via `contractimport!`.

## TODOs & Potential Enhancements

*   **Enhanced Authorization**: Explore more granular control over function calls, potentially differentiating permissions for various operations.
*   **Multi-Signer Support**: Investigate mechanisms to allow different types of signers (e.g., for automated strategies) or multi-signature control over the account.
*   **Event Granularity**: Consider adding more specific events for different stages within the `deposit_usdc` and `redeem` functions to provide better off-chain tracking.
*   **Expanded Error Handling**: Add more specific error variants to `AccountError` to cover potential issues during interactions with the `Router` (e.g., `Router.provide_liquidity` failing) or token contracts (e.g., insufficient balance/allowance before calling router).
*   **Gas Optimization**: Review token approval and transfer patterns for potential gas savings. For instance, `deposit_usdc` involves the owner transferring to the account, then the account approving the router.
*   **Direct Swap Functionality**: Consider adding a convenience function to perform direct token swaps via the `Router`, abstracting the underlying router calls for the user.
*   **Router Interface Robustness**: Ensure the imported `hoops_router.wasm` path is reliable for deployment and consider alternatives if needed.
