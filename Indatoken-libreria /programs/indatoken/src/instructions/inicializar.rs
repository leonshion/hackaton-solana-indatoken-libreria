use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};
use crate::state::Plataforma;

pub const SUPPLY_INICIAL: u64 = 1_000_000 * 1_000_000; // 1M INDATOKEN (6 decimales)

/// Instrucción 1 — Inicializar plataforma
/// - Crea PDA Plataforma
/// - Crea mint INDATOKEN (6 decimales, authority = PDA)
/// - Crea treasury ATA
/// - Acuña 1,000,000 INDATOKEN al treasury
pub fn handler(ctx: Context<InicializarPlataforma>, nombre: String) -> Result<()> {
    let bump = ctx.bumps.plataforma;

    let p         = &mut ctx.accounts.plataforma;
    p.authority   = *ctx.accounts.authority.key;
    p.mint        = ctx.accounts.mint.key();
    p.nombre      = nombre.clone();
    p.total_articulos = 0;
    p.total_autores   = 0;
    p.bump        = bump;

    // CPI firmado por el PDA plataforma
    let seeds  = &[b"plataforma".as_ref(), &[bump]];
    let signer = &[&seeds[..]];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint:      ctx.accounts.mint.to_account_info(),
                to:        ctx.accounts.treasury.to_account_info(),
                authority: ctx.accounts.plataforma.to_account_info(),
            },
            signer,
        ),
        SUPPLY_INICIAL,
    )?;

    msg!("✅ IndaSOCIAL '{}' inicializada.", nombre);
    msg!("🪙 Mint: {}", ctx.accounts.mint.key());
    msg!("🏦 1,000,000 INDATOKEN → treasury.");
    Ok(())
}

#[derive(Accounts)]
pub struct InicializarPlataforma<'info> {
    #[account(
        init, payer = authority,
        space  = 8 + Plataforma::INIT_SPACE,
        seeds  = [b"plataforma"], bump
    )]
    pub plataforma: Account<'info, Plataforma>,

    #[account(
        init, payer = authority,
        mint::decimals  = 6,
        mint::authority = plataforma,
        seeds = [b"indatoken_mint"], bump
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init, payer = authority,
        associated_token::mint      = mint,
        associated_token::authority = plataforma,
    )]
    pub treasury: Account<'info, TokenAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program:            Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program:           Program<'info, System>,
    pub rent:                     Sysvar<'info, Rent>,
}
