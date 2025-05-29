# Hoops Account Deployer Contract

The `hoops-account-deployer` contract is a simple utility responsible for deploying new instances of the `hoops-account` contract.

## Core Functionality

The contract exposes a single function:

*   `deploy_account(env: Env, owner: Address, router: Address, wasm_hash: BytesN<32>, salt: BytesN<32>) -> Address`:
    *   Requires authorization from the `owner` address.
    *   Deploys a new contract instance using the provided `wasm_hash` (which should be the hash of the `hoops-account` contract WASM) and a `salt` for deterministic address generation.
    *   The `owner` address is used as the deployer's address, meaning the deployed `hoops-account` contract will be associated with this owner.
    *   The `router` address parameter is currently included for potential future use or interface compatibility but is not actively used in the deployment logic of the `hoops-account` itself by this deployer. The `hoops-account` takes the router address in its `initialize` function.
    *   Returns the `Address` of the newly deployed `hoops-account` contract.

## Dependencies

*   `soroban-sdk`

## Usage

To deploy a new `hoops-account`:

1.  Obtain the WASM hash of the compiled `hoops-account` contract.
2.  The intended owner of the new account calls `deploy_account` on this `Deployer` contract, providing their own address as `owner`, the `router` address (for future use, can be a placeholder if not immediately needed by the account's initialization logic via deployer), the `wasm_hash`, and a unique `salt`.
3.  The `Deployer` will then deploy the `hoops-account` contract. The caller (owner) will then need to call the `initialize` function on the newly deployed account contract, passing the `owner` and `router` addresses.

## TODOs & Potential Enhancements

*   **Initialization Call**: Consider if the deployer should also call the `initialize` function on the newly deployed `hoops-account` contract as part of the `deploy_account` transaction. This would simplify the setup process for the user, though it would require the `Deployer` contract to have the `hoops-account` client or interface.
*   **Router Parameter Usage**: Clarify or implement the intended use of the `router` parameter. If it's meant to be passed to the `hoops-account` during its initialization, the `deploy_account` function would need to be updated to call the `initialize` function of the new account contract.
*   **Version Management**: For more complex systems, add upgrade features for managing different versions of the `hoops-account` WASM hash.
*   **Access Control**: Currently, any address can call `deploy_account` as long as they provide an `owner` that authorizes the call. Depending on the desired trust model, this might be too open. Access control could be added to restrict who can use this deployer.
