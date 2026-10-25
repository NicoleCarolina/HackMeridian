//! Interfaz mínima del router/agregador de swaps (compatible con Soroswap).
//!
//! La plataforma solo depende de este trait: en producción apunta al router
//! de Soroswap; en tests se registra un router simulado (`MockRouter`).
//! VERIFICAR en testnet: firmas exactas y dirección del router vigente.
use soroban_sdk::{contractclient, Address, Env, Vec};

#[contractclient(name = "RouterClient")]
pub trait Router {
    /// Cantidades esperadas a lo largo de `path` para `amount_in`.
    fn router_get_amounts_out(env: Env, amount_in: i128, path: Vec<Address>) -> Vec<i128>;

    /// Dirección del par/pool que recibirá el input (el router lo pulls de `to`).
    /// VERIFICAR: nombre y firma exactos contra el router de Soroswap en testnet.
    fn router_pair_for(env: Env, token_a: Address, token_b: Address) -> Address;

    /// Swap exacto de entrada. `to` paga el input y recibe el output.
    fn swap_exact_tokens_for_tokens(
        env: Env,
        amount_in: i128,
        amount_out_min: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Vec<i128>;
}
