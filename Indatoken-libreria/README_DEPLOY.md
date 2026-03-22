# 🚀 Guía de Deploy — Solana Playground (Devnet)

## Paso 1 — Abrir Solana Playground

Ve a → **[beta.solpg.io](https://beta.solpg.io)**

---

## Paso 2 — Crear proyecto Anchor

1. Click en **"Create a new project"**
2. Nombre: `indatoken`
3. Selecciona **Anchor (Rust)**
4. Click **Create**

---

## Paso 3 — Pegar el código

El programa está dividido en módulos. En Solana Playground,
pega **todo el contenido** de `programs/indatoken/src/lib.rs`
en el archivo `src/lib.rs` del proyecto.

> Solana Playground acepta el programa completo en un solo archivo.
> Para eso, copia también el contenido de los módulos inlined
> (ver `lib.rs` — ya tiene los imports correctos).

---

## Paso 4 — Compilar

Click en el ícono 🔨 **Build** (panel izquierdo).

Si compila sin errores verás:
```
Build successful
```

---

## Paso 5 — Obtener SOL de prueba

1. En el panel inferior, busca tu wallet de Playground
2. Click en **Airdrop** (te da 2 SOL de Devnet gratis)
3. Verifica: `solana balance` → debe mostrar `2 SOL`

---

## Paso 6 — Deploy en Devnet

Click en el ícono 🚀 **Deploy**.

Verás el **Program ID** generado, por ejemplo:
```
Program Id: 59FujaiRkNmPiZTZGZRNuTNKeaDgaTFjWkvkr1QgN6Mz
```

**Copia ese Program ID** y reemplázalo en `declare_id!()` del `lib.rs`.

---

## Paso 7 — Probar instrucciones

En Solana Playground puedes llamar cada instrucción desde la UI:

| Instrucción            | Qué hace                                      |
|------------------------|-----------------------------------------------|
| `inicializar_plataforma` | Crea el mint INDATOKEN + treasury             |
| `registrar_autor`      | Crea perfil del autor (Fase 1)                |
| `publicar_articulo`    | Publica artículo on-chain + reward si Fase 2  |
| `otorgar_autor_top`    | Asigna badge Top + 5 INDATOKEN bonus          |
| `leer_articulo_top`    | Lector paga 1 INDA (80% autor / 20% treasury) |
| `transferir_indatoken` | Envío libre entre wallets (eventos, etc.)     |

---

## Paso 8 — Ver transacciones en Solana Explorer

Cada tx tiene un signature. Puedes verla en:
```
https://explorer.solana.com/tx/<SIGNATURE>?cluster=devnet
```

---

## Recursos oficiales

- Anchor Docs: https://www.anchor-lang.com/docs
- Solana Deploy Docs: https://solana.com/docs/programs/deploying
- Phantom Connect: https://docs.phantom.com/phantom-connect
- Solana Playground: https://beta.solpg.io

---

*IndaSOCIAL · INDATOKEN · Solana LATAM Hackathon | WayLearn*
