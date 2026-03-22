# Handoff: S2c-Web-V2 — TerraGenesis Branding + Simplification

## Context

La web app tiene branding de HardTrust (11 secciones, 1,069 líneas, lenguaje genérico de device framework). Transformar en una app simple y enfocada de verificación TerraGenesis (~5 secciones, ~500 líneas).

## Spec

Read `docs/specs/s2c-web-v2-branding.spec.md` — contiene paleta de colores, copy exacto, y estructura de página target.

## Task (un solo branch: `feat/s2c-web-v2-branding`)

### 1. Actualizar paleta de colores (styles.css)

Reemplazar TODAS las CSS custom properties en `:root` con las definidas en la spec (teal/purple/yellow de biotexturas). Actualizar el body background gradient. Reemplazar TODAS las referencias hardcodeadas a orange (`#ff895b`, `rgba(255,137,91,...)`) por los nuevos colores. Actualizar gradientes de botones para usar `--accent` / `--accent-light`.

### 2. Actualizar index.html

- Title: `TerraGenesis — Provenance for Every Observation`
- Meta description: `Verify biological data provenance from TerraScope microscopes on-chain.`

### 3. Eliminar secciones (App.jsx)

- **Eliminar** el array `storyPoints` y toda su sección de rendering (story-grid)
- **Eliminar** el array `workflowSteps` y toda su sección de rendering (workflow-grid)
- **Eliminar** la sección status-grid (contract address + chain pills) — mover esa info al footer

### 4. Rebranding copy (App.jsx)

Reemplazar TODO el copy según las tablas de la spec:
- Header: eyebrow → `"TerraGenesis"`, h1 → `"Provenance for every observation."`
- Hero: kicker, h2, lead paragraph, stat cards — todo el copy nuevo está en la spec
- Secciones: "devices" → "microscopes", "register a device" → "register a TerraScope", etc.
- Zero ocurrencias de "HardTrust" en UI visible (solo footer)

### 5. Reestructurar página (App.jsx)

- **Agregar** 3 inline steps después del hero (Capture → Sign → Verify) — una sola fila horizontal, NO cards
- **Mover** la sección de capture verify ANTES del registry (es la acción primaria)
- **Colapsar** el formulario de register: solo renderizar si `isAuthorizedAttester` es true, detrás de un toggle/botón
- **Mover** wallet info (address, balance, chain) dentro del panel de register colapsado
- **Eliminar** los párrafos explicativos sobre keccak256 en la sección register

### 6. Simplificar resultados de verificación (App.jsx)

Reemplazar los 9+ status pills con:
1. **Big verdict card** (full width): fondo verde + "VERIFIED — from TerraScope [serial]" O fondo rojo + "NOT VERIFIED"
2. **Summary row** (3 items): "Files ✓ | Hash ✓ | On-chain ✓"
3. **Detalles colapsables** (cerrado por default): signer address, script hash, binary hash, per-file hashes. Usar `<details><summary>` o toggle state.

### 7. Agregar footer (App.jsx + styles.css)

```html
<footer>TerraGenesis · A biotexturas project · Built on HardTrust · Contract: {addr} · Chain: {chainId}</footer>
```

### Validar

- `cd web && npm run dev` — app arranca sin errores
- Zero "HardTrust" visible en la UI (solo footer footnote)
- ~5 secciones, no 11
- Verify es la primera sección después del hero
- Big verdict card visible (VERIFIED verde / NOT VERIFIED rojo)
- Colores son teal/purple, no orange
- `grep -ri "hardtrust" web/src/` solo retorna contract.js (ABI) y el footer

## Branch

`feat/s2c-web-v2-branding`

## Acceptance

- Page simplificada (~5 secciones)
- Branding TerraGenesis completo (colores, copy, estructura)
- Verify como acción primaria con big verdict card
- Register colapsado detrás de toggle attester-only
- Footer con contract/chain info
