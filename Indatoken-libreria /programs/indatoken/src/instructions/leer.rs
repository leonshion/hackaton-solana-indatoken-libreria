use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{Articulo, AutorCuenta, Plataforma};
use crate::errors::IndaError;
use crate::instructions::autor::FACTOR;

pub const COSTO_LEER:      u64 = 1;   // 1 INDATOKEN para leer artículo premium
pub const SPLIT_AUTOR:     u64 = 80;  // 80% al autor
pub const SPLIT_PLATAFORMA:u64 = 20;  // 20% al treasury

// ─────────────────────────────────────────────────────────────
// INSTRUCCIÓN 5 — Leer Artículo Top
//
// El lector paga 1 INDATOKEN:
//   → 0.8 INDA al autor
//   → 0.2 INDA al treasury IndaSOCIAL
// Solo aplica para artículos de Autores Top (es_premium = true)
// ─────────────────────────────────────────────────────────────
pub fn leer(ctx: Context<LeerArticuloTop>) -> Result<()> {
    require!(ctx.accounts.articulo.es_premium, IndaError::ArticuloNoEsTop);

    let total        = COSTO_LEER  * FACTOR;          // 1_000_000
    let para_autor   = total * SPLIT_AUTOR     / 100;  //   800_000
    let para_plataforma = total * SPLIT_PLATAFORMA / 100; // 200_000

    // 80% → autor
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from:      ctx.accounts.lector_token_account.to_account_info(),
                to:        ctx.accounts.autor_token_account.to_account_info(),
                authority: ctx.accounts.lector.to_account_info(),
            },
        ),
        para_autor,
    )?;

    // 20% → treasury
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from:      ctx.accounts.lector_token_account.to_account_info(),
                to:        ctx.accounts.treasury.to_account_info(),
                authority: ctx.accounts.lector.to_account_info(),
            },
        ),
        para_plataforma,
    )?;

    // Incrementar engagement del artículo on-chain
    ctx.accounts.articulo.engagement += 1;

    msg!("📖 Artículo desbloqueado.");
    msg!("💸 0.8 INDA → autor | 0.2 INDA → treasury IndaSOCIAL");
    Ok(())
}

#[derive(Accounts)]
pub struct LeerArticuloTop<'info> {
    #[account(mut)]
    pub articulo: Account<'info, Articulo>,

    #[account(
        seeds = [b"autor", autor_wallet.key().as_ref()], bump
    )]
    pub autor_cuenta: Account<'info, AutorCuenta>,

    /// CHECK: referencia para derivar PDA del autor
    pub autor_wallet: AccountInfo<'info>,

    #[account(mut)]
    pub autor_token_account: Account<'info, TokenAccount>,

    #[account(mut, seeds = [b"plataforma"], bump)]
    pub plataforma: Account<'info, Plataforma>,

    #[account(mut)]
    pub treasury: Account<'info, TokenAccount>,

    #[account(mut)]
    pub lector_token_account: Account<'info, TokenAccount>,

    pub lector:        Signer<'info>,
    pub token_program: Program<'info, Token>,
}
