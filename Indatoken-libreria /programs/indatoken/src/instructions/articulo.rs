use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::state::{Articulo, AutorCuenta, Plataforma};
use crate::errors::IndaError;
use crate::instructions::autor::{FACTOR, UMBRAL_ESTABLECIDO};

pub const REWARD_PUBLICAR: u64 = 1; // +1 INDA al publicar (Fase 2+)

// ─────────────────────────────────────────────────────────────
// INSTRUCCIÓN 4 — Publicar Artículo
//
// Fase 1 (1–14 artículos): gratis, sin recompensa
// Fase 2 (15+): gratis + 1 INDATOKEN del treasury al autor
// Fase 3 (Top): igual que Fase 2 + artículo marcado es_premium=true
// ─────────────────────────────────────────────────────────────
pub fn publicar(
    ctx:       Context<PublicarArticulo>,
    titulo:    String,
    categoria: String,
    resumen:   String,
    url:       String,
) -> Result<()> {
    require!(titulo.len() <= 120, IndaError::TituloDemasiado);

    let es_top = ctx.accounts.autor_cuenta.es_top;

    // Guardar artículo on-chain
    let art       = &mut ctx.accounts.articulo;
    art.autor      = *ctx.accounts.wallet.key;
    art.titulo     = titulo.clone();
    art.categoria  = categoria;
    art.resumen    = resumen;
    art.url        = url;
    art.es_premium = es_top; // premium si es Autor Top
    art.engagement = 0;
    art.timestamp  = Clock::get()?.unix_timestamp;
    art.bump       = ctx.bumps.articulo;

    // Actualizar contador del autor
    let autor             = &mut ctx.accounts.autor_cuenta;
    autor.total_articulos += 1;

    // ¿Alcanza Fase 2?
    if autor.total_articulos >= UMBRAL_ESTABLECIDO && !autor.es_establecido {
        autor.es_establecido = true;
        msg!("🎉 '{}' es ahora Autor Establecido (Fase 2). Recompensas activas.", autor.nombre);
    }

    // Si Fase 2 o superior → enviar 1 INDATOKEN
    if autor.es_establecido {
        let bump   = ctx.accounts.plataforma.bump;
        let seeds  = &[b"plataforma".as_ref(), &[bump]];
        let signer = &[&seeds[..]];

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
            REWARD_PUBLICAR * FACTOR,
        )?;

        autor.tokens_ganados += REWARD_PUBLICAR;
        msg!("🪙 +1 INDATOKEN → '{}'.", autor.nombre);
    } else {
        msg!(
            "📝 Artículo #{} publicado. Faltan {} para recompensas.",
            autor.total_articulos,
            UMBRAL_ESTABLECIDO - autor.total_articulos
        );
    }

    ctx.accounts.plataforma.total_articulos += 1;
    msg!("📰 '{}' publicado on-chain.", titulo);
    Ok(())
}

#[derive(Accounts)]
#[instruction(titulo: String)]
pub struct PublicarArticulo<'info> {
    #[account(
        init, payer = wallet,
        space = 8 + Articulo::INIT_SPACE,
        seeds = [b"articulo", wallet.key().as_ref(), titulo.as_bytes()],
        bump
    )]
    pub articulo: Account<'info, Articulo>,

    #[account(
        mut,
        seeds = [b"autor", wallet.key().as_ref()], bump
    )]
    pub autor_cuenta: Account<'info, AutorCuenta>,

    #[account(mut, seeds = [b"plataforma"], bump)]
    pub plataforma: Account<'info, Plataforma>,

    #[account(mut)]
    pub treasury: Account<'info, TokenAccount>,

    #[account(
        init_if_needed, payer = wallet,
        associated_token::mint      = mint,
        associated_token::authority = wallet,
    )]
    pub autor_token_account: Account<'info, TokenAccount>,

    #[account(seeds = [b"indatoken_mint"], bump)]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub wallet: Signer<'info>,

    pub token_program:            Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program:           Program<'info, System>,
    pub rent:                     Sysvar<'info, Rent>,
}
