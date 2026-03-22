
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};

declare_id!("Ghkiecht9YHzPkZCT3GYBhYc2nZgKfY1XnjZcKnmQbek");

// ── Constantes INDATOKEN ────────────────────────────────────────
const SUPPLY_INICIAL:      u64 = 1_000_000 * 1_000_000; // 1M tokens (6 decimales)
const FACTOR:              u64 = 1_000_000;              // multiplicador 6 decimales
const UMBRAL_ESTABLECIDO:  u64 = 15;                     // artículos para Fase 2
const REWARD_PUBLICAR:     u64 = 1;                      // +1 INDA al publicar (Fase 2+)
const REWARD_TOP:          u64 = 5;                      // +5 INDA al otorgar badge Top
const COSTO_LEER:          u64 = 1;                      // 1 INDA para leer premium
const SPLIT_AUTOR:         u64 = 80;                     // 80% del pago al autor
const SPLIT_PLATAFORMA:    u64 = 20;                     // 20% al treasury

// ═══════════════════════════════════════════════════════════════
// PROGRAMA PRINCIPAL
// ═══════════════════════════════════════════════════════════════
#[program]
pub mod indatoken {
    use super::*;

    // ── 1. Inicializar plataforma ───────────────────────────────
    // Crea el PDA Plataforma, el mint INDATOKEN (SPL) y el treasury.
    // Acuña 1,000,000 INDATOKEN al treasury.
    // Llamar UNA SOLA VEZ desde la wallet admin de IndaSOCIAL.
    pub fn inicializar_plataforma(
        ctx:    Context<InicializarPlataforma>,
        nombre: String,
    ) -> Result<()> {
        let bump = ctx.bumps.plataforma;

        let p             = &mut ctx.accounts.plataforma;
        p.authority       = *ctx.accounts.authority.key;
        p.mint            = ctx.accounts.mint.key();
        p.nombre          = nombre.clone();
        p.total_articulos = 0;
        p.total_autores   = 0;
        p.bump            = bump;

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

        msg!("✅ IndaSOCIAL '{}' inicializada en Solana.", nombre);
        msg!("🪙 Mint INDATOKEN: {}", ctx.accounts.mint.key());
        msg!("🏦 1,000,000 INDATOKEN acuñados al treasury.");
        Ok(())
    }

    // ── 2. Registrar autor ──────────────────────────────────────
    // Cualquier wallet puede registrarse.
    // Empieza en Fase 1 (sin recompensas todavía).
    pub fn registrar_autor(
        ctx:    Context<RegistrarAutor>,
        nombre: String,
    ) -> Result<()> {
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

    // ── 3. Publicar artículo ────────────────────────────────────
    // Fase 1 (0–14): gratis, sin recompensa.
    // Fase 2 (15+):  gratis + 1 INDATOKEN del treasury.
    // Fase 3 (Top):  igual Fase 2 + artículo marcado es_premium.
    pub fn publicar_articulo(
        ctx:       Context<PublicarArticulo>,
        titulo:    String,
        categoria: String,
        resumen:   String,
        url:       String,
    ) -> Result<()> {
        require!(titulo.len() <= 120, IndaError::TituloDemasiado);

        let es_top = ctx.accounts.autor_cuenta.es_top;

        // Guardar artículo on-chain
        let art        = &mut ctx.accounts.articulo;
        art.autor      = *ctx.accounts.wallet.key;
        art.titulo     = titulo.clone();
        art.categoria  = categoria;
        art.resumen    = resumen;
        art.url        = url;
        art.es_premium = es_top;
        art.engagement = 0;
        art.timestamp  = Clock::get()?.unix_timestamp;
        art.bump       = ctx.bumps.articulo;

        // Actualizar contador del autor
        let autor             = &mut ctx.accounts.autor_cuenta;
        autor.total_articulos += 1;

        // ¿Alcanza el umbral de Fase 2?
        if autor.total_articulos >= UMBRAL_ESTABLECIDO && !autor.es_establecido {
            autor.es_establecido = true;
            msg!("🎉 '{}' es Autor Establecido (Fase 2). ¡Recompensas activas!", autor.nombre);
        }

        // Si es Fase 2 o superior → enviar 1 INDATOKEN
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

    // ── 4. Otorgar badge Autor Top + 5 INDATOKEN bonus ─────────
    // SOLO la authority de la plataforma puede llamar esta ix.
    // Activa es_top=true y envía 5 INDA de bonus al autor.
    // Los artículos futuros del autor quedan marcados premium.
    pub fn otorgar_autor_top(ctx: Context<OtorgarAutorTop>) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ctx.accounts.plataforma.authority,
            IndaError::NoAutorizado
        );

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
            REWARD_TOP * FACTOR,
        )?;

        let autor             = &mut ctx.accounts.autor_cuenta;
        autor.es_top          = true;
        autor.tokens_ganados += REWARD_TOP;

        msg!("⭐ '{}' es ahora Autor Top de IndaSOCIAL!", autor.nombre);
        msg!("🪙 +5 INDATOKEN de bonus enviados.");
        msg!("🔒 Sus artículos requieren 1 INDA para ser leídos.");
        Ok(())
    }

    // ── 5. Leer artículo premium ────────────────────────────────
    // El lector paga 1 INDATOKEN:
    //   → 0.8 INDA al autor (80%)
    //   → 0.2 INDA al treasury IndaSOCIAL (20%)
    // Solo funciona para artículos con es_premium = true.
    pub fn leer_articulo_top(ctx: Context<LeerArticuloTop>) -> Result<()> {
        require!(ctx.accounts.articulo.es_premium, IndaError::ArticuloNoEsTop);

        let total           = COSTO_LEER * FACTOR;              // 1_000_000
        let para_autor      = total * SPLIT_AUTOR / 100;        //   800_000
        let para_plataforma = total * SPLIT_PLATAFORMA / 100;   //   200_000

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

        // Engagement on-chain
        ctx.accounts.articulo.engagement += 1;

        msg!("📖 Artículo desbloqueado exitosamente.");
        msg!("💸 0.8 INDA → autor | 0.2 INDA → treasury IndaSOCIAL.");
        Ok(())
    }

    // ── 6. Transferir INDATOKEN ─────────────────────────────────
    // Transferencia libre entre wallets.
    // Usos: pagar eventos, accesos exclusivos, pagos P2P en Phantom.
    pub fn transferir_indatoken(
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
}

// ═══════════════════════════════════════════════════════════════
// ERRORES CUSTOM
// ═══════════════════════════════════════════════════════════════
#[error_code]
pub enum IndaError {
    #[msg("No autorizado para realizar esta acción.")]
    NoAutorizado,

    #[msg("Cantidad inválida — debe ser mayor a 0.")]
    CantidadInvalida,

    #[msg("El artículo no pertenece a un Autor Top.")]
    ArticuloNoEsTop,

    #[msg("El autor aún no alcanzó los 15 artículos requeridos.")]
    AutorNoEstablecido,

    #[msg("Nombre demasiado largo — máximo 80 caracteres.")]
    NombreDemasiado,

    #[msg("Título demasiado largo — máximo 120 caracteres.")]
    TituloDemasiado,
}

// ═══════════════════════════════════════════════════════════════
// STRUCTS DE DATOS (State)
// ═══════════════════════════════════════════════════════════════

/// Cuenta global de IndaSOCIAL — PDA: seeds = [b"plataforma"]
#[account]
#[derive(InitSpace)]
pub struct Plataforma {
    pub authority:       Pubkey,  // wallet admin IndaSOCIAL
    pub mint:            Pubkey,  // INDATOKEN mint address
    #[max_len(100)]
    pub nombre:          String,
    pub total_articulos: u64,
    pub total_autores:   u64,
    pub bump:            u8,
}

/// Perfil on-chain de cada autor — PDA: seeds = [b"autor", wallet.key()]
#[account]
#[derive(InitSpace)]
pub struct AutorCuenta {
    pub wallet:          Pubkey,
    #[max_len(80)]
    pub nombre:          String,
    pub total_articulos: u64,
    pub tokens_ganados:  u64,
    pub es_establecido:  bool,   // true desde artículo #15
    pub es_top:          bool,   // true cuando authority otorga badge Top
    pub bump:            u8,
}

/// Artículo on-chain — PDA: seeds = [b"articulo", wallet.key(), titulo.as_bytes()]
#[account]
#[derive(InitSpace)]
pub struct Articulo {
    pub autor:      Pubkey,
    #[max_len(120)]
    pub titulo:     String,
    #[max_len(60)]
    pub categoria:  String,
    #[max_len(280)]
    pub resumen:    String,
    #[max_len(200)]
    pub url:        String,
    pub es_premium: bool,   // true si autor es Top (lectores pagan)
    pub engagement: u64,    // contador de lecturas on-chain
    pub timestamp:  i64,    // unix timestamp de publicación
    pub bump:       u8,
}

// ═══════════════════════════════════════════════════════════════
// CONTEXTOS DE INSTRUCCIONES (Accounts)
// ═══════════════════════════════════════════════════════════════

#[derive(Accounts)]
pub struct InicializarPlataforma<'info> {
    #[account(
        init, payer = authority,
        space = 8 + Plataforma::INIT_SPACE,
        seeds = [b"plataforma"], bump
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

#[derive(Accounts)]
pub struct LeerArticuloTop<'info> {
    #[account(mut)]
    pub articulo: Account<'info, Articulo>,

    #[account(seeds = [b"autor", autor_wallet.key().as_ref()], bump)]
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

#[derive(Accounts)]
pub struct TransferirIndatoken<'info> {
    #[account(mut)]
    pub origen: Account<'info, TokenAccount>,

    #[account(mut)]
    pub destino: Account<'info, TokenAccount>,

    pub remitente:     Signer<'info>,
    pub token_program: Program<'info, Token>,
}
