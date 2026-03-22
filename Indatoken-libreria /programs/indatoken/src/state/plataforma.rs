use anchor_lang::prelude::*;

/// Cuenta global de IndaSOCIAL — PDA: seeds = [b"plataforma"]
#[account]
#[derive(InitSpace)]
pub struct Plataforma {
    pub authority:        Pubkey,   // wallet admin IndaSOCIAL
    pub mint:             Pubkey,   // INDATOKEN mint address
    #[max_len(100)]
    pub nombre:           String,
    pub total_articulos:  u64,
    pub total_autores:    u64,
    pub bump:             u8,
}
