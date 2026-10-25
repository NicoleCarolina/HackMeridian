//! Interfaz mínima del router AMM de Soroswap.
//!
//! Firmas y nombres verificados contra `soroswap/core` (branch main) en el
//! contrato `contracts/router/src/lib.rs`. El router real devuelve
//! `Result<T, CombinedRouterError>`; `#[contractclient]` desenvuelve el `Ok`
//! en la llamada cross-contract, por lo que estas firmas ven solo `T`.
//!
//! Dirección en testnet (soroswap/core public/testnet.contracts.json):
//!   CCJUD55AG6W5HAI5LRVNKAE5WDP5XGZBUDS5WNTIVDU7O264UZZE7BRD
//! Dirección en mainnet:
//!   CAG5LRYQ5JVEUI5TEID72EYOVX44TTUJT5BQR2J6J77FH65PCCFAJDDH
//!
//! En tests se registra un router simulado (`MockRouter`) con la misma firma.
use soroban_sdk::{contractclient, Address, Env, Vec};

#[contractclient(name = "RouterClient")]
pub trait Router {
    /// Cantidades esperadas a lo largo de `path` para `amount_in`.
    /// Soroswap: `router_get_amounts_out(e, amount_in, path) -> Vec<i128>`.
    fn router_get_amounts_out(env: Env, amount_in: i128, path: Vec<Address>) -> Vec<i128>;

    /// Dirección determinista del par para `(token_a, token_b)`. El router AMM
    /// extrae el input de `to` hacia este par antes del swap, por eso el
    /// contrato DCA pre-autoriza `transfer(to, pair, amount_in)` sobre el token
    /// de entrada con `env.authorize_as_current_contract(...)`.
    /// Soroswap: `router_pair_for(e, token_a, token_b) -> Address`.
    fn router_pair_for(env: Env, token_a: Address, token_b: Address) -> Address;

    /// Swap exacto de entrada. `to` paga el input y recibe el output.
    /// Soroswap: `swap_exact_tokens_for_tokens(e, amount_in, amount_out_min, path, to, deadline) -> Vec<i128>`.
    fn swap_exact_tokens_for_tokens(
        env: Env,
        amount_in: i128,
        amount_out_min: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Vec<i128>;
}
