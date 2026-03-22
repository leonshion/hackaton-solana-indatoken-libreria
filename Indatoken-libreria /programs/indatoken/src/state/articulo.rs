use anchor_lang::prelude::*;

/// Artículo on-chain — PDA: seeds = [b"articulo", autor_wallet, titulo.as_bytes()]
#[account]
#[derive(InitSpace)]
pub struct Articulo {
    pub autor:       Pubkey,
    #[max_len(120)]
    pub titulo:      String,
    #[max_len(60)]
    pub categoria:   String,
    #[max_len(280)]
    pub resumen:     String,
    #[max_len(200)]
    pub url:         String,
    pub es_premium:  bool,    // true si el autor es Top (pago para leer)
    pub engagement:  u64,     // contador de lecturas on-chain
    pub timestamp:   i64,     // unix timestamp de publicación
    pub bump:        u8,
}
