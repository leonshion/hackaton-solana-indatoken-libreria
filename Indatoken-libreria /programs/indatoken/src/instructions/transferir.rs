use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::errors::IndaError;
use crate::instructions::autor::FACTOR;

// ─────────────────────────────────────────────────────────────
// INSTRUCCIÓN 6 — Transferir INDATOKEN
//
// Usos: pagar eventos, accesos exclusivos, wallet a wallet
// ─────────────────────────────────────────────────────────────
pub fn transferir(
    ctx:      Context<TransferirIndatoken>,
    cantidad: u64,
    concepto: String,
) -> Result<()> {
    require!(cantidad > 0, IndaError::CantidadInvalida);

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from:      ctx.accounts.origen.to_account_info(),
                to:        ctx.accounts.destino.to_account_info(),
                authority: ctx.accounts.remitente.to_account_info(),
            },
        ),
        cantidad * FACTOR,
    )?;

    msg!("💸 {} INDATOKEN transferidos.", cantidad);
    msg!("📌 Concepto: {}", concepto);
    Ok(())
}

#[derive(Accounts)]
pub struct TransferirIndatoken<'info> {
    #[account(mut)]
    pub origen: Account<'info, TokenAccount>,

    #[account(mut)]
    pub destino: Account<'info, TokenAccount>,

    pub remitente:     Signer<'info>,
    pub token_program: Program<'info, Token>,
}
