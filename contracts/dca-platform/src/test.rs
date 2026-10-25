#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, Vec,
};

// ---- Router simulado: precio fijo + descuento de ejecución configurable ----
#[contract]
pub struct MockRouter;

#[contractimpl]
impl MockRouter {
    pub fn __constructor(env: Env, num: i128, den: i128) {
        env.storage().instance().set(&1u32, &(num, den));
        env.storage().instance().set(&2u32, &0u32); // descuento en bps
    }
    pub fn set_discount_bps(env: Env, bps: u32) {
        env.storage().instance().set(&2u32, &bps);
    }
    pub fn router_pair_for(env: Env, _a: Address, _b: Address) -> Address {
        env.current_contract_address() // el "par" simulado es el propio router
    }
    pub fn router_get_amounts_out(env: Env, amount_in: i128, path: Vec<Address>) -> Vec<i128> {
        let (n, d): (i128, i128) = env.storage().instance().get(&1u32).unwrap();
        soroban_sdk::vec![&env, amount_in, amount_in * n / d].slice(0..path.len())
    }
    pub fn swap_exact_tokens_for_tokens(
        env: Env,
        amount_in: i128,
        amount_out_min: i128,
        path: Vec<Address>,
        to: Address,
        _deadline: u64,
    ) -> Vec<i128> {
        to.require_auth();
        let (n, d): (i128, i128) = env.storage().instance().get(&1u32).unwrap();
        let disc: u32 = env.storage().instance().get(&2u32).unwrap();
        let out = amount_in * n / d * (10_000 - disc as i128) / 10_000;
        assert!(out >= amount_out_min, "router: min out");
        let me = env.current_contract_address();
        TokenClient::new(&env, &path.get(0).unwrap()).transfer(&to, &me, &amount_in);
        TokenClient::new(&env, &path.get(1).unwrap()).transfer(&me, &to, &out);
        soroban_sdk::vec![&env, amount_in, out]
    }
}

struct Ctx<'a> {
    env: Env,
    dca: DcaPlatformClient<'a>,
    router: MockRouterClient<'a>,
    usdc: Address,
    xlm: Address,
    treasury: Address,
    admin: Address,
    alice: Address,
}

const AMOUNT: i128 = 1_000_000;

fn setup<'a>() -> Ctx<'a> {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000);
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let alice = Address::generate(&env);
    let usdc = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let xlm = env.register_stellar_asset_contract_v2(admin.clone()).address();

    let router_id = env.register(MockRouter, (8_i128, 1_i128)); // 1 USDC = 8 XLM
    StellarAssetClient::new(&env, &xlm).mint(&router_id, &1_000_000_000);
    StellarAssetClient::new(&env, &usdc).mint(&alice, &100_000_000);

    let dca_id = env.register(DcaPlatform, (&admin, &treasury, &router_id, 50_u32)); // 0,5 %
    let dca = DcaPlatformClient::new(&env, &dca_id);
    let router = MockRouterClient::new(&env, &router_id);
    Ctx { env, dca, router, usdc, xlm, treasury, admin, alice }
}

fn bal(c: &Ctx, token: &Address, who: &Address) -> i128 {
    TokenClient::new(&c.env, token).balance(who)
}

fn new_plan(c: &Ctx, buys: u32, slippage: u32) -> u64 {
    c.dca.create_plan(&c.alice, &c.usdc, &c.xlm, &AMOUNT, &3_600, &buys, &slippage)
}

#[test]
fn create_plan_escrows_deposit() {
    let c = setup();
    let id = new_plan(&c, 5, 100);
    assert_eq!(id, 1);
    assert_eq!(bal(&c, &c.usdc, &c.dca.address), 5 * AMOUNT);
    assert_eq!(bal(&c, &c.usdc, &c.alice), 100_000_000 - 5 * AMOUNT);
    let p = c.dca.get_plan(&id);
    assert_eq!(p.remaining, 5);
    assert_eq!(c.dca.get_user_plans(&c.alice).len(), 1);
}

#[test]
fn execute_charges_fee_and_swaps() {
    let c = setup();
    let id = new_plan(&c, 3, 100);
    let out = c.dca.execute(&id);
    // fee 0,5 % = 5_000 ; neto 995_000 ; 1 USDC = 8 XLM
    assert_eq!(bal(&c, &c.usdc, &c.treasury), 5_000);
    assert_eq!(c.dca.total_fees(&c.usdc), 5_000);
    assert_eq!(out, 995_000 * 8);
    assert_eq!(bal(&c, &c.xlm, &c.alice), 995_000 * 8);
    assert_eq!(c.dca.get_plan(&id).remaining, 2);
    // Lo que queda en escrow es exactamente lo de las compras pendientes
    assert_eq!(bal(&c, &c.usdc, &c.dca.address), 2 * AMOUNT);
}

#[test]
fn execute_before_interval_fails_then_works() {
    let c = setup();
    let id = new_plan(&c, 3, 100);
    c.dca.execute(&id);
    assert_eq!(c.dca.try_execute(&id), Err(Ok(Error::TooEarly)));
    c.env.ledger().set_timestamp(1_000 + 3_599);
    assert_eq!(c.dca.try_execute(&id), Err(Ok(Error::TooEarly)));
    c.env.ledger().set_timestamp(1_000 + 3_600);
    c.dca.execute(&id);
    assert_eq!(c.dca.get_plan(&id).remaining, 1);
}

#[test]
fn plan_completes_after_all_buys() {
    let c = setup();
    let id = new_plan(&c, 2, 100);
    c.dca.execute(&id);
    c.env.ledger().set_timestamp(10_000);
    c.dca.execute(&id);
    c.env.ledger().set_timestamp(20_000);
    assert_eq!(c.dca.try_execute(&id), Err(Ok(Error::PlanNotActive)));
    assert_eq!(bal(&c, &c.usdc, &c.dca.address), 0);
}

#[test]
fn cancel_refunds_remaining() {
    let c = setup();
    let id = new_plan(&c, 4, 100);
    c.dca.execute(&id);
    let refund = c.dca.cancel(&id);
    assert_eq!(refund, 3 * AMOUNT);
    assert_eq!(bal(&c, &c.usdc, &c.dca.address), 0);
    assert_eq!(c.dca.get_plan(&id).remaining, 0);
    c.env.ledger().set_timestamp(50_000);
    assert_eq!(c.dca.try_execute(&id), Err(Ok(Error::PlanNotActive)));
    assert_eq!(c.dca.try_cancel(&id), Err(Ok(Error::PlanNotActive)));
}

#[test]
fn slippage_protection_reverts_swap() {
    let c = setup();
    let id = new_plan(&c, 2, 100); // tolera 1 %
    c.router.set_discount_bps(&300); // el router ejecuta 3 % peor que la cotización
    assert!(c.dca.try_execute(&id).is_err());
    // todo se revierte: el plan sigue intacto y sin comisión cobrada
    assert_eq!(c.dca.get_plan(&id).remaining, 2);
    assert_eq!(c.dca.total_fees(&c.usdc), 0);
    assert_eq!(bal(&c, &c.usdc, &c.dca.address), 2 * AMOUNT);
    c.router.set_discount_bps(&50); // 0,5 % < 1 %: pasa
    c.dca.execute(&id);
}

#[test]
fn fee_is_capped_and_admin_only() {
    let c = setup();
    assert_eq!(c.dca.try_set_fee_bps(&201), Err(Ok(Error::FeeTooHigh)));
    c.dca.set_fee_bps(&200);
    assert_eq!(c.dca.get_config().fee_bps, 200);
    let id = new_plan(&c, 1, 100);
    c.dca.execute(&id);
    assert_eq!(c.dca.total_fees(&c.usdc), 20_000); // 2 % de 1_000_000
    assert_eq!(c.dca.get_config().admin, c.admin);
}

#[test]
fn pause_blocks_create_and_execute_but_not_cancel() {
    let c = setup();
    let id = new_plan(&c, 2, 100);
    c.dca.set_paused(&true);
    assert_eq!(c.dca.try_execute(&id), Err(Ok(Error::Paused)));
    assert_eq!(
        c.dca.try_create_plan(&c.alice, &c.usdc, &c.xlm, &AMOUNT, &3_600, &1, &100),
        Err(Ok(Error::Paused))
    );
    assert_eq!(c.dca.cancel(&id), 2 * AMOUNT);
}

#[test]
fn rejects_invalid_params() {
    let c = setup();
    let a = &c.alice;
    assert_eq!(c.dca.try_create_plan(a, &c.usdc, &c.xlm, &0, &3_600, &1, &100), Err(Ok(Error::InvalidParams)));
    assert_eq!(c.dca.try_create_plan(a, &c.usdc, &c.xlm, &AMOUNT, &10, &1, &100), Err(Ok(Error::InvalidParams)));
    assert_eq!(c.dca.try_create_plan(a, &c.usdc, &c.xlm, &AMOUNT, &3_600, &0, &100), Err(Ok(Error::InvalidParams)));
    assert_eq!(c.dca.try_create_plan(a, &c.usdc, &c.usdc, &AMOUNT, &3_600, &1, &100), Err(Ok(Error::SameAsset)));
    assert_eq!(c.dca.try_create_plan(a, &c.usdc, &c.xlm, &AMOUNT, &3_600, &1, &2_000), Err(Ok(Error::InvalidSlippage)));
    assert_eq!(c.dca.try_get_plan(&99), Err(Ok(Error::PlanNotFound)));
}
