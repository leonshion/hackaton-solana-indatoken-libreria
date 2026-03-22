use anchor_lang::prelude::*;

/// Perfil on-chain de cada autor — PDA: seeds = [b"autor", wallet.key()]
///
/// Fase 1 — Nuevo      (0–14 artículos): gratis, sin recompensa
/// Fase 2 — Establecido (15+ artículos): gratis + 1 INDATOKEN por artículo
/// Fase 3 — Autor Top  (asignado por authority): +5 INDA bonus,
///           sus artículos se vuelven premium (lectores pagan 1 INDA)
#[account]
#[derive(InitSpace)]
pub struct AutorCuenta {
    pub wallet:          Pubkey,
    #[max_len(80)]
    pub nombre:          String,
    pub total_articulos: u64,
    pub tokens_ganados:  u64,
    pub es_establecido:  bool,   // true desde artículo #15
    pub es_top:          bool,   // true cuando authority otorga badge
    pub bump:            u8,
}
