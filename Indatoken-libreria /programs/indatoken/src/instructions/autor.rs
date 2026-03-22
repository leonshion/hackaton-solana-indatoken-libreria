use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::state::{AutorCuenta, Plataforma};
use crate::errors::IndaError;

pub const FACTOR:             u64 = 1_000_000; // 6 decimales
pub const UMBRAL_ESTABLECIDO: u64 = 15;        // artículos para Fase 2
pub const REWARD_TOP:         u64 = 5;         // INDA bonus al otorgar Top

// ─────────────────────────────────────────────────────────────
// INSTRUCCIÓN 2 — Registrar Autor (Fase 1)
// ─────────────────────────────────────────────────────────────
pub fn registrar(ctx: Context<RegistrarAutor>, nombre: String) -> Result<()> {
    require!(nombre.len() <= 80, IndaError::NombreDemasiado);

    let autor             = &mut ctx.accounts.autor;
    autor.wallet          = *ctx.accounts.wallet.key;
    autor.nombre          = nombre.clone();
    autor.total_articulos = 0;
    autor.tokens_ganados  = 0;
    autor.es_establecido  = false;
    autor.es_top          = false;
    autor.bump            = ctx.bumps.autor;

    ctx.accounts.plataforma.total_autores += 1;

    msg!("👤 Autor '{}' registrado — Fase 1 (0/{}).", nombre, UMBRAL_ESTABLECIDO);
    Ok(())
}

#[derive(Accounts)]
pub struct RegistrarAutor<'info> {
    #[account(
        init, payer = wallet,
        space = 8 + AutorCuenta::INIT_SPACE,
        seeds = [b"autor", wallet.key().as_ref()], bump
    )]
    pub autor: Account<'info, AutorCuenta>,

    #[account(mut, seeds = [b"plataforma"], bump)]
    pub plataforma: Account<'info, Plataforma>,

    #[account(mut)]
    pub wallet: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// ─────────────────────────────────────────────────────────────
// INSTRUCCIÓN 3 — Otorgar badge Autor Top + 5 INDATOKEN
// Solo la authority de la plataforma puede llamar esta instrucción
// ─────────────────────────────────────────────────────────────
pub fn otorgar_top(ctx: Context<OtorgarAutorTop>) -> Result<()> {
    require!(
        ctx.accounts.authority.key() == ctx.accounts.plataforma.authority,
        IndaError::NoAutorizado
    );

    let bump   = ctx.accounts.plataforma.bump;
    let seeds  = &[b"plataforma".as_ref(), &[bump]];
    let signer = &[&seeds[..]];

    // Enviar 5 INDATOKEN del treasury al autor
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from:      ctx.accounts.treasury.to_account_info(),
                to:        ctx.accounts.autor_token_account.to_account_info(),
                authority: ctx.accounts.plataforma.to_account_info(),
            },
            signer,
        ),
        REWARD_TOP * FACTOR,
    )?;

    let autor             = &mut ctx.accounts.autor_cuenta;
    autor.es_top          = true;
    autor.tokens_ganados += REWARD_TOP;

    msg!("⭐ '{}' es ahora Autor Top!", autor.nombre);
    msg!("🪙 +5 INDATOKEN de bonus enviados.");
    msg!("🔒 Sus artículos ahora requieren 1 INDA para ser leídos.");
    Ok(())
}

#[derive(Accounts)]
pub struct OtorgarAutorTop<'info> {
    #[account(mut, seeds = [b"plataforma"], bump)]
    pub plataforma: Account<'info, Plataforma>,

    #[account(mut)]
    pub treasury: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"autor", autor_wallet.key().as_ref()], bump
    )]
    pub autor_cuenta: Account<'info, AutorCuenta>,

    #[account(
        init_if_needed, payer = authority,
        associated_token::mint      = mint,
        associated_token::authority = autor_wallet,
    )]
    pub autor_token_account: Account<'info, TokenAccount>,

    /// CHECK: solo referencia para derivar PDA del autor
    pub autor_wallet: AccountInfo<'info>,

    #[account(seeds = [b"indatoken_mint"], bump)]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program:            Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program:           Program<'info, System>,
    pub rent:                     Sysvar<'info, Rent>,
}
