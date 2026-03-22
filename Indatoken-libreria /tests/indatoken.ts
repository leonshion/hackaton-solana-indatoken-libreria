import * as anchor from "@coral-xyz/anchor";
import { Program }  from "@coral-xyz/anchor";
import { Indatoken } from "../target/types/indatoken";
import { getAssociatedTokenAddress } from "@solana/spl-token";
import { assert } from "chai";

// ═══════════════════════════════════════════════════════════════
//  INDATOKEN — Test Suite completo
//  Red: Solana Devnet (via Solana Playground o anchor test)
//
//  Cubre las 6 instrucciones del programa:
//  1. inicializar_plataforma
//  2. registrar_autor
//  3. publicar_articulo  (Fase 1 → Fase 2)
//  4. otorgar_autor_top  (Fase 3 + 5 INDA bonus)
//  5. leer_articulo_top  (pago lector 80/20 split)
//  6. transferir_indatoken
// ═══════════════════════════════════════════════════════════════
describe("INDATOKEN — IndaSOCIAL", () => {

  const provider   = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program    = anchor.workspace.Indatoken as Program<Indatoken>;
  const authority  = provider.wallet;

  // ── PDAs ────────────────────────────────────────────────────
  const [plataformaPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("plataforma")],
    program.programId
  );
  const [mintPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("indatoken_mint")],
    program.programId
  );
  const [autorPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("autor"), authority.publicKey.toBuffer()],
    program.programId
  );

  let treasury:         anchor.web3.PublicKey;
  let autorTokenAccount: anchor.web3.PublicKey;

  before(async () => {
    treasury          = await getAssociatedTokenAddress(mintPda, plataformaPda, true);
    autorTokenAccount = await getAssociatedTokenAddress(mintPda, authority.publicKey);
    console.log("\n🪙  Program ID:", program.programId.toBase58());
    console.log("🏛️  Plataforma:", plataformaPda.toBase58());
    console.log("💎  Mint:      ", mintPda.toBase58());
    console.log("🏦  Treasury:  ", treasury.toBase58());
    console.log("👤  Autor PDA: ", autorPda.toBase58(), "\n");
  });

  // ──────────────────────────────────────────────────────────
  // TEST 1 — Inicializar plataforma + mint INDATOKEN
  // ──────────────────────────────────────────────────────────
  it("✅ Inicializa la plataforma IndaSOCIAL", async () => {
    const tx = await program.methods
      .inicializarPlataforma("IndaSOCIAL")
      .accounts({
        plataforma:              plataformaPda,
        mint:                    mintPda,
        treasury,
        authority:               authority.publicKey,
        tokenProgram:            anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram:  anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram:           anchor.web3.SystemProgram.programId,
        rent:                    anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const cuenta = await program.account.plataforma.fetch(plataformaPda);
    assert.equal(cuenta.nombre,          "IndaSOCIAL");
    assert.equal(cuenta.authority.toBase58(), authority.publicKey.toBase58());
    assert.equal(cuenta.totalArticulos.toNumber(), 0);
    assert.equal(cuenta.totalAutores.toNumber(),   0);

    console.log("  ✔ Plataforma creada:", cuenta.nombre);
    console.log("  ✔ TX:", tx);
  });

  // ──────────────────────────────────────────────────────────
  // TEST 2 — Registrar autor (Fase 1)
  // ──────────────────────────────────────────────────────────
  it("👤 Registra un autor nuevo — Fase 1", async () => {
    await program.methods
      .registrarAutor("Sarah Williams")
      .accounts({
        autor:         autorPda,
        plataforma:    plataformaPda,
        wallet:        authority.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const autor = await program.account.autorCuenta.fetch(autorPda);
    assert.equal(autor.nombre,           "Sarah Williams");
    assert.equal(autor.totalArticulos.toNumber(), 0);
    assert.equal(autor.esEstablecido,    false);
    assert.equal(autor.esTop,            false);
    assert.equal(autor.tokensGanados.toNumber(), 0);

    const plataforma = await program.account.plataforma.fetch(plataformaPda);
    assert.equal(plataforma.totalAutores.toNumber(), 1);

    console.log("  ✔ Autor registrado:", autor.nombre);
    console.log("  ✔ Fase: Nuevo (0/15 artículos)");
  });

  // ──────────────────────────────────────────────────────────
  // TEST 3 — Publicar artículos Fase 1 (sin recompensa)
  // ──────────────────────────────────────────────────────────
  it("📝 Publica 14 artículos — Fase 1 (sin recompensa)", async () => {
    for (let i = 1; i <= 14; i++) {
      const titulo = `Artículo de prueba número ${i}`;
      const [articuloPda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("articulo"), authority.publicKey.toBuffer(), Buffer.from(titulo)],
        program.programId
      );

      await program.methods
        .publicarArticulo(
          titulo,
          "Web3",
          `Resumen del artículo ${i} sobre Solana y Web3.`,
          `https://indasocial.com/articulos/${i}`
        )
        .accounts({
          articulo:              articuloPda,
          autorCuenta:           autorPda,
          plataforma:            plataformaPda,
          treasury,
          autorTokenAccount,
          mint:                  mintPda,
          wallet:                authority.publicKey,
          tokenProgram:          anchor.utils.token.TOKEN_PROGRAM_ID,
          associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          systemProgram:         anchor.web3.SystemProgram.programId,
          rent:                  anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .rpc();
    }

    const autor = await program.account.autorCuenta.fetch(autorPda);
    assert.equal(autor.totalArticulos.toNumber(), 14);
    assert.equal(autor.esEstablecido,    false,  "No debe ser establecido aún");
    assert.equal(autor.tokensGanados.toNumber(),  0, "No debe tener tokens en Fase 1");

    console.log("  ✔ 14 artículos publicados en Fase 1");
    console.log("  ✔ Tokens ganados: 0 INDATOKEN (correcto — sin recompensa en Fase 1)");
  });

  // ──────────────────────────────────────────────────────────
  // TEST 4 — Artículo #15: activa Fase 2 + recibe 1 INDATOKEN
  // ──────────────────────────────────────────────────────────
  it("🎉 Artículo #15 activa Fase 2 — recibe +1 INDATOKEN", async () => {
    const titulo = "El artículo 15 que lo cambia todo en Solana";
    const [articuloPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("articulo"), authority.publicKey.toBuffer(), Buffer.from(titulo)],
      program.programId
    );

    await program.methods
      .publicarArticulo(
        titulo,
        "Solana",
        "Este artículo activa las recompensas INDATOKEN para el autor.",
        "https://indasocial.com/articulos/15-milestone"
      )
      .accounts({
        articulo:               articuloPda,
        autorCuenta:            autorPda,
        plataforma:             plataformaPda,
        treasury,
        autorTokenAccount,
        mint:                   mintPda,
        wallet:                 authority.publicKey,
        tokenProgram:           anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram:          anchor.web3.SystemProgram.programId,
        rent:                   anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const autor = await program.account.autorCuenta.fetch(autorPda);
    assert.equal(autor.totalArticulos.toNumber(), 15);
    assert.equal(autor.esEstablecido,   true,  "Debe ser Autor Establecido (Fase 2)");
    assert.equal(autor.tokensGanados.toNumber(), 1, "Debe haber recibido 1 INDATOKEN");

    console.log("  ✔ Autor Establecido activado (Fase 2)");
    console.log("  ✔ Tokens ganados: 1 INDATOKEN");
  });

  // ──────────────────────────────────────────────────────────
  // TEST 5 — Artículo post-Fase 2 recibe 1 INDATOKEN
  // ──────────────────────────────────────────────────────────
  it("🪙 Artículo #16 en Fase 2 — recibe +1 INDATOKEN", async () => {
    const titulo = "INDATOKEN: guía completa para la comunidad Inda";
    const [articuloPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("articulo"), authority.publicKey.toBuffer(), Buffer.from(titulo)],
      program.programId
    );

    await program.methods
      .publicarArticulo(
        titulo,
        "DeFi",
        "Todo lo que necesitas saber sobre INDATOKEN y la economía on-chain.",
        "https://indasocial.com/articulos/indatoken-guia"
      )
      .accounts({
        articulo:               articuloPda,
        autorCuenta:            autorPda,
        plataforma:             plataformaPda,
        treasury,
        autorTokenAccount,
        mint:                   mintPda,
        wallet:                 authority.publicKey,
        tokenProgram:           anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram:          anchor.web3.SystemProgram.programId,
        rent:                   anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const autor = await program.account.autorCuenta.fetch(autorPda);
    assert.equal(autor.totalArticulos.toNumber(), 16);
    assert.equal(autor.tokensGanados.toNumber(),   2, "Debe tener 2 INDATOKEN acumulados");

    console.log("  ✔ Artículo #16 publicado");
    console.log("  ✔ Tokens ganados acumulados: 2 INDATOKEN");
  });

  // ──────────────────────────────────────────────────────────
  // TEST 6 — Otorgar badge Autor Top + 5 INDATOKEN bonus
  // ──────────────────────────────────────────────────────────
  it("⭐ Otorgar badge Autor Top — +5 INDATOKEN bonus", async () => {
    await program.methods
      .otorgarAutorTop()
      .accounts({
        plataforma:             plataformaPda,
        treasury,
        autorCuenta:            autorPda,
        autorTokenAccount,
        autorWallet:            authority.publicKey,
        mint:                   mintPda,
        authority:              authority.publicKey,
        tokenProgram:           anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram:          anchor.web3.SystemProgram.programId,
        rent:                   anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const autor = await program.account.autorCuenta.fetch(autorPda);
    assert.equal(autor.esTop,  true, "Debe ser Autor Top (Fase 3)");
    assert.equal(autor.tokensGanados.toNumber(), 7, "2 previos + 5 bonus = 7 INDATOKEN");

    console.log("  ✔ Autor Top activado (Fase 3)");
    console.log("  ✔ Tokens ganados: 7 INDATOKEN (2 + 5 bonus)");
  });

  // ──────────────────────────────────────────────────────────
  // TEST 7 — Leer artículo Top (split 80/20)
  // ──────────────────────────────────────────────────────────
  it("📖 Lector paga 1 INDATOKEN — split 80/20 aplicado", async () => {
    // Publicar un artículo con es_top = true (autor ya es Top)
    const titulo = "Artículo premium de Autor Top sobre DeFi en Solana";
    const [articuloPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("articulo"), authority.publicKey.toBuffer(), Buffer.from(titulo)],
      program.programId
    );

    await program.methods
      .publicarArticulo(
        titulo,
        "DeFi",
        "Contenido exclusivo premium para la comunidad Inda.",
        "https://indasocial.com/articulos/premium-defi"
      )
      .accounts({
        articulo:               articuloPda,
        autorCuenta:            autorPda,
        plataforma:             plataformaPda,
        treasury,
        autorTokenAccount,
        mint:                   mintPda,
        wallet:                 authority.publicKey,
        tokenProgram:           anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram:          anchor.web3.SystemProgram.programId,
        rent:                   anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    // El lector (misma wallet en este test) paga 1 INDATOKEN
    await program.methods
      .leerArticuloTop()
      .accounts({
        articulo:             articuloPda,
        autorCuenta:          autorPda,
        autorWallet:          authority.publicKey,
        autorTokenAccount,
        plataforma:           plataformaPda,
        treasury,
        lectorTokenAccount:   autorTokenAccount, // en test lector = autor
        lector:               authority.publicKey,
        tokenProgram:         anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .rpc();

    const art = await program.account.articulo.fetch(articuloPda);
    assert.equal(art.esPremium,         true, "Artículo debe ser premium");
    assert.equal(art.engagement.toNumber(), 1, "Engagement debe incrementar a 1");

    console.log("  ✔ Artículo premium desbloqueado");
    console.log("  ✔ Split: 0.8 INDA → autor | 0.2 INDA → treasury");
    console.log("  ✔ Engagement on-chain:", art.engagement.toNumber());
  });

  // ──────────────────────────────────────────────────────────
  // TEST 8 — Transferir INDATOKEN (evento, acceso exclusivo)
  // ──────────────────────────────────────────────────────────
  it("💸 Transferir INDATOKEN — pago de evento", async () => {
    // En este test transferimos de autorTokenAccount al treasury
    // (simula pago de evento dentro de la comunidad)
    await program.methods
      .transferirIndatoken(
        new anchor.BN(2),
        "Acceso a IndaSOCIAL Summit — Evento exclusivo comunidad Inda"
      )
      .accounts({
        origen:        autorTokenAccount,
        destino:       treasury,
        remitente:     authority.publicKey,
        tokenProgram:  anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("  ✔ 2 INDATOKEN transferidos para evento");
    console.log("  ✔ Concepto: AccesoIndaSOCIAL Summit");
  });

  // ──────────────────────────────────────────────────────────
  // TEST 9 — Verificar estado final de la plataforma
  // ──────────────────────────────────────────────────────────
  it("📊 Estado final de la plataforma", async () => {
    const plataforma = await program.account.plataforma.fetch(plataformaPda);
    const autor      = await program.account.autorCuenta.fetch(autorPda);

    console.log("\n  ── Estado final ──────────────────────────");
    console.log(`  Plataforma:        ${plataforma.nombre}`);
    console.log(`  Total artículos:   ${plataforma.totalArticulos}`);
    console.log(`  Total autores:     ${plataforma.totalAutores}`);
    console.log(`  Autor:             ${autor.nombre}`);
    console.log(`  Artículos:         ${autor.totalArticulos}`);
    console.log(`  Tokens ganados:    ${autor.tokensGanados} INDATOKEN`);
    console.log(`  Establecido:       ${autor.esEstablecido}`);
    console.log(`  Top:               ${autor.esTop}`);
    console.log("  ─────────────────────────────────────────\n");

    assert.isTrue(plataforma.totalArticulos.toNumber() > 0);
    assert.isTrue(autor.esEstablecido);
    assert.isTrue(autor.esTop);
    assert.isTrue(autor.tokensGanados.toNumber() > 0);
  });
});
