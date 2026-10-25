#[soroban_sdk::contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Monto, intervalo o número de compras inválido
    InvalidParams = 1,
    /// Los dos activos del plan son el mismo
    SameAsset = 2,
    /// Slippage fuera de rango (máx. 10 %)
    InvalidSlippage = 3,
    /// El plan no existe
    PlanNotFound = 4,
    /// El plan ya no tiene compras pendientes
    PlanNotActive = 5,
    /// Todavía no pasó el intervalo desde la última compra
    TooEarly = 6,
    /// La plataforma está en pausa (cancelar sigue permitido)
    Paused = 7,
    /// La comisión supera el máximo permitido (2 %)
    FeeTooHigh = 8,
    /// El swap devolvió menos del mínimo aceptado (slippage)
    SlippageExceeded = 9,
    /// El router no devolvió una cotización válida
    BadQuote = 10,
    /// El usuario alcanzó el máximo de planes activos
    TooManyPlans = 11,
    /// Overflow aritmético
    MathOverflow = 12,
}
