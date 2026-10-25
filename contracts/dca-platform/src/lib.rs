#![no_std]
//! DCA Platform: compras recurrentes (USDC <-> XLM) con comisión por ejecución.
use soroban_sdk::{
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    contract, contractevent, contractimpl, contracttype, token, vec, Address, Env, IntoVal, Symbol,
    Vec,
};

mod error;
pub mod router;

pub use error::Error;
use router::RouterClient;

const DAY_IN_LEDGERS: u32 = 17_280;
const BUMP_THRESHOLD: u32 = 30 * DAY_IN_LEDGERS;
const BUMP_TO: u32 = 120 * DAY_IN_LEDGERS;

pub const MAX_FEE_BPS: u32 = 200; // 2 %
pub const MAX_SLIPPAGE_BPS: u32 = 1_000; // 10 %
pub const MIN_INTERVAL: u64 = 60; // segundos
pub const MAX_BUYS: u32 = 365;
pub const MAX_PLANS_PER_USER: u32 = 50;
const BPS: i128 = 10_000;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Treasury,
    Router,
    FeeBps,
    Paused,
    NextId,
    Plan(u64),
    UserPlans(Address),
    Fees(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Plan {
    pub owner: Address,
    pub token_in: Address,
    pub token_out: Address,
    pub amount_per_buy: i128,
    pub interval: u64,
    pub remaining: u32,
    pub next_exec: u64,
    pub slippage_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub admin: Address,
    pub treasury: Address,
    pub router: Address,
    pub fee_bps: u32,
    pub paused: bool,
}

#[contractevent(topics = ["created"])]
pub struct PlanCreated {
    #[topic]
    pub owner: Address,
    pub plan_id: u64,
    pub deposit: i128,
}

#[contractevent(topics = ["executed"])]
pub struct PlanExecuted {
    #[topic]
    pub owner: Address,
    pub plan_id: u64,
    pub amount_in: i128,
    pub fee: i128,
    pub amount_out: i128,
}

#[contractevent(topics = ["cancelled"])]
pub struct PlanCancelled {
    #[topic]
    pub owner: Address,
    pub plan_id: u64,
    pub refund: i128,
}

#[contract]
pub struct DcaPlatform;

#[contractimpl]
impl DcaPlatform {
    pub fn __constructor(
        env: Env,
        admin: Address,
        treasury: Address,
        router: Address,
        fee_bps: u32,
    ) -> Result<(), Error> {
        if fee_bps > MAX_FEE_BPS {
            return Err(Error::FeeTooHigh);
        }
        let s = env.storage().instance();
        s.set(&DataKey::Admin, &admin);
        s.set(&DataKey::Treasury, &treasury);
        s.set(&DataKey::Router, &router);
        s.set(&DataKey::FeeBps, &fee_bps);
        s.set(&DataKey::Paused, &false);
        s.set(&DataKey::NextId, &1u64);
        Ok(())
    }

    // ---------------------------------------------------------------- usuario

    /// Crea un plan y recibe el depósito completo (monto * nº de compras).
    /// La primera compra puede ejecutarse de inmediato.
    pub fn create_plan(
        env: Env,
        owner: Address,
        token_in: Address,
        token_out: Address,
        amount_per_buy: i128,
        interval: u64,
        num_buys: u32,
        slippage_bps: u32,
    ) -> Result<u64, Error> {
        owner.require_auth();
        Self::bump_instance(&env);
        if Self::paused(&env) {
            return Err(Error::Paused);
        }
        if amount_per_buy <= 0 || interval < MIN_INTERVAL || num_buys == 0 || num_buys > MAX_BUYS {
            return Err(Error::InvalidParams);
        }
        if token_in == token_out {
            return Err(Error::SameAsset);
        }
        if slippage_bps > MAX_SLIPPAGE_BPS {
            return Err(Error::InvalidSlippage);
        }
        let deposit = amount_per_buy
            .checked_mul(num_buys as i128)
            .ok_or(Error::MathOverflow)?;

        let mut mine: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::UserPlans(owner.clone()))
            .unwrap_or(vec![&env]);
        if mine.len() >= MAX_PLANS_PER_USER {
            return Err(Error::TooManyPlans);
        }

        let id: u64 = env.storage().instance().get(&DataKey::NextId).unwrap_or(1);
        env.storage().instance().set(&DataKey::NextId, &(id + 1));

        let plan = Plan {
            owner: owner.clone(),
            token_in: token_in.clone(),
            token_out,
            amount_per_buy,
            interval,
            remaining: num_buys,
            next_exec: env.ledger().timestamp(),
            slippage_bps,
        };
        Self::save_plan(&env, id, &plan);
        mine.push_back(id);
        let key = DataKey::UserPlans(owner.clone());
        env.storage().persistent().set(&key, &mine);
        env.storage().persistent().extend_ttl(&key, BUMP_THRESHOLD, BUMP_TO);

        token::Client::new(&env, &token_in).transfer(
            &owner,
            &env.current_contract_address(),
            &deposit,
        );
        PlanCreated { owner, plan_id: id, deposit }.publish(&env);
        Ok(id)
    }

    /// Ejecuta una compra del plan. Cualquiera puede llamarla (bot/keeper).
    /// Devuelve el monto recibido por el dueño del plan.
    pub fn execute(env: Env, plan_id: u64) -> Result<i128, Error> {
        Self::bump_instance(&env);
        if Self::paused(&env) {
            return Err(Error::Paused);
        }
        let mut plan = Self::load_plan(&env, plan_id)?;
        if plan.remaining == 0 {
            return Err(Error::PlanNotActive);
        }
        let now = env.ledger().timestamp();
        if now < plan.next_exec {
            return Err(Error::TooEarly);
        }

        // Efectos primero (checks-effects-interactions).
        plan.remaining -= 1;
        plan.next_exec = now.saturating_add(plan.interval);
        Self::save_plan(&env, plan_id, &plan);

        let fee_bps: u32 = env.storage().instance().get(&DataKey::FeeBps).unwrap_or(0);
        let fee = plan
            .amount_per_buy
            .checked_mul(fee_bps as i128)
            .ok_or(Error::MathOverflow)?
            / BPS;
        let net = plan.amount_per_buy - fee;

        let me = env.current_contract_address();
        let tin = token::Client::new(&env, &plan.token_in);
        if fee > 0 {
            let treasury: Address = env.storage().instance().get(&DataKey::Treasury).unwrap();
            tin.transfer(&me, &treasury, &fee);
            Self::add_fees(&env, &plan.token_in, fee);
        }

        // Swap vía router. NOTA: la cotización se toma en la misma transacción;
        // en mainnet conviene contrastarla con un oráculo (p. ej. Reflector).
        let router_addr: Address = env.storage().instance().get(&DataKey::Router).unwrap();
        let router = RouterClient::new(&env, &router_addr);
        let path = vec![&env, plan.token_in.clone(), plan.token_out.clone()];
        let quoted = router
            .router_get_amounts_out(&net, &path)
            .last()
            .ok_or(Error::BadQuote)?;
        if quoted <= 0 {
            return Err(Error::BadQuote);
        }
        let min_out = quoted * (BPS - plan.slippage_bps as i128) / BPS;
        let deadline = now + 300;
        // El router toma el input de `to` (este contrato) y lo envía al par:
        // hay que pre-autorizar esa sub-invocación de `transfer`.
        let pair = router.router_pair_for(&plan.token_in, &plan.token_out);
        env.authorize_as_current_contract(vec![
            &env,
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: plan.token_in.clone(),
                    fn_name: Symbol::new(&env, "transfer"),
                    args: (me.clone(), pair, net).into_val(&env),
                },
                sub_invocations: vec![&env],
            }),
        ]);
        let amounts = router.swap_exact_tokens_for_tokens(&net, &min_out, &path, &me, &deadline);
        let out = amounts.last().ok_or(Error::BadQuote)?;
        if out < min_out {
            return Err(Error::SlippageExceeded);
        }

        token::Client::new(&env, &plan.token_out).transfer(&me, &plan.owner, &out);
        PlanExecuted {
            owner: plan.owner,
            plan_id,
            amount_in: net,
            fee,
            amount_out: out,
        }
        .publish(&env);
        Ok(out)
    }

    /// Cancela el plan y devuelve el saldo no ejecutado. Siempre permitido.
    pub fn cancel(env: Env, plan_id: u64) -> Result<i128, Error> {
        Self::bump_instance(&env);
        let mut plan = Self::load_plan(&env, plan_id)?;
        plan.owner.require_auth();
        if plan.remaining == 0 {
            return Err(Error::PlanNotActive);
        }
        let refund = plan
            .amount_per_buy
            .checked_mul(plan.remaining as i128)
            .ok_or(Error::MathOverflow)?;
        plan.remaining = 0;
        Self::save_plan(&env, plan_id, &plan);
        token::Client::new(&env, &plan.token_in).transfer(
            &env.current_contract_address(),
            &plan.owner,
            &refund,
        );
        PlanCancelled { owner: plan.owner, plan_id, refund }.publish(&env);
        Ok(refund)
    }

    // ---------------------------------------------------------------- lectura

    pub fn get_plan(env: Env, plan_id: u64) -> Result<Plan, Error> {
        Self::load_plan(&env, plan_id)
    }

    pub fn get_user_plans(env: Env, owner: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::UserPlans(owner))
            .unwrap_or(vec![&env])
    }

    /// Comisiones acumuladas (en `token`) enviadas a la tesorería.
    pub fn total_fees(env: Env, token: Address) -> i128 {
        env.storage().persistent().get(&DataKey::Fees(token)).unwrap_or(0)
    }

    pub fn get_config(env: Env) -> Config {
        let s = env.storage().instance();
        Config {
            admin: s.get(&DataKey::Admin).unwrap(),
            treasury: s.get(&DataKey::Treasury).unwrap(),
            router: s.get(&DataKey::Router).unwrap(),
            fee_bps: s.get(&DataKey::FeeBps).unwrap_or(0),
            paused: s.get(&DataKey::Paused).unwrap_or(false),
        }
    }

    // ------------------------------------------------------------------ admin

    pub fn set_fee_bps(env: Env, fee_bps: u32) -> Result<(), Error> {
        Self::require_admin(&env);
        if fee_bps > MAX_FEE_BPS {
            return Err(Error::FeeTooHigh);
        }
        env.storage().instance().set(&DataKey::FeeBps, &fee_bps);
        Ok(())
    }

    pub fn set_treasury(env: Env, treasury: Address) {
        Self::require_admin(&env);
        env.storage().instance().set(&DataKey::Treasury, &treasury);
    }

    pub fn set_router(env: Env, router: Address) {
        Self::require_admin(&env);
        env.storage().instance().set(&DataKey::Router, &router);
    }

    /// Pausa de emergencia: bloquea crear y ejecutar; cancelar sigue activo.
    pub fn set_paused(env: Env, paused: bool) {
        Self::require_admin(&env);
        env.storage().instance().set(&DataKey::Paused, &paused);
    }

    // ---------------------------------------------------------------- helpers

    fn require_admin(env: &Env) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        Self::bump_instance(env);
    }

    fn bump_instance(env: &Env) {
        env.storage().instance().extend_ttl(BUMP_THRESHOLD, BUMP_TO);
    }

    fn paused(env: &Env) -> bool {
        env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
    }

    fn load_plan(env: &Env, id: u64) -> Result<Plan, Error> {
        let key = DataKey::Plan(id);
        let plan: Plan = env.storage().persistent().get(&key).ok_or(Error::PlanNotFound)?;
        env.storage().persistent().extend_ttl(&key, BUMP_THRESHOLD, BUMP_TO);
        Ok(plan)
    }

    fn save_plan(env: &Env, id: u64, plan: &Plan) {
        let key = DataKey::Plan(id);
        env.storage().persistent().set(&key, plan);
        env.storage().persistent().extend_ttl(&key, BUMP_THRESHOLD, BUMP_TO);
    }

    fn add_fees(env: &Env, token: &Address, fee: i128) {
        let key = DataKey::Fees(token.clone());
        let cur: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage().persistent().set(&key, &(cur + fee));
        env.storage().persistent().extend_ttl(&key, BUMP_THRESHOLD, BUMP_TO);
    }
}

#[cfg(test)]
mod test;
