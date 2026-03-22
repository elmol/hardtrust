# S2c-Web-V2 — TerraGenesis Branding + Simplification

> **Status:** Draft
> **Story:** S2c — Web Portal
> **Depends on:** S2c-Web-V1 (implemented)

---

## Context

The web app was built with HardTrust branding (11 sections, 1,069 lines, generic device framework language). It needs to become a **simple, focused TerraGenesis verification app** — reflecting the biotexturas identity and the DeSci microscopy narrative.

### Brand Sources

- **biotexturas.org** — "Connecting people, machines, and ecosystems"
- **Spirit:** "Build it! Grow it! Hack it!" — DIY science, maker culture, open source
- **Logo:** Organic lichen/microbial form in a circle, teal green (#5BA89A) with darker borders (#3D7A6E)
- **Microscopy clusters:** Purple (#3B1F70) + yellow (#E8D44D) + green (#4CAF50)
- **TerraScope:** DIY robotic microscope, Raspberry Pi camera, XY+Z stage, captures biological samples

---

## 1. Page Structure — Simplification

### From 11 sections → 5 sections

Current page is a developer control panel. Target is a **single-purpose verification app**.

```
[Header: TerraGenesis | wallet connect (subtle)]

[HERO:
  "Prove your microscopy data is real."
  2-sentence pitch | "Verify a capture" CTA
  Single stat: "N registered TerraScopes"
]

[HOW IT WORKS: single horizontal row, 3 steps with icons
  Capture → Sign → Verify
]

[VERIFY A CAPTURE: (PRIMARY action — promoted from last to first)
  Upload image + metadata + sign.json
  Image preview | Verify button
  BIG VERDICT: "VERIFIED" or "NOT VERIFIED" (full-width, green/red)
  Expandable technical details underneath
]

[REGISTERED TERRASCOPES: (secondary, compact grid)
  Microscope cards with address, serial, attestation date
]

[Footer: contract address, chain, "A biotexturas project · Built on HardTrust"]
```

### What to REMOVE

| Section | Action | Why |
|---------|--------|-----|
| Story Points (3 cards) | **DELETE** | Hero copy does this job in one sentence |
| Workflow Steps (4 cards) | **REPLACE** with 3 inline icons | Too verbose for hackathon |
| Status grid (contract/chain) | **MOVE to footer** | Technical detail, not user-facing |
| Register form | **COLLAPSE** behind toggle | Only visible if wallet is authorized attester |
| Wallet box (address/balance/chain) | **MOVE inside register panel** | Only relevant for attester |
| Register explanatory paragraphs | **DELETE** | Engineers read the code |

### Verification Results — Simplification

Current: 9+ status pills + per-file breakdown = information overload.

Target:
1. **Big verdict card** (full width): "VERIFIED — from TerraScope #ba807092" or "NOT VERIFIED" with green/red background
2. **Summary row** (3 items): "Files match ✓ | Content hash match ✓ | On-chain verified ✓"
3. **Expandable details** (collapsed by default): signer address, script hash, binary hash, per-file hashes

---

## 2. Naming Rules

| Name | Usage | Never |
|------|-------|-------|
| **TerraGenesis** | Product name, web app title, headers | Never say "HardTrust" in UI |
| **TerraScope** | The physical microscope hardware only | Don't use as software name |
| **biotexturas** | Footer attribution only | Not in main headers |
| **HardTrust** | Footer footnote: "Built on HardTrust" | Never in user-facing UI |

---

## 3. Color Palette

### CSS Custom Properties

```css
:root {
  /* Backgrounds — dark, scientific instrument feel */
  --bg-deep:        #0B1622;
  --bg-surface:     #111D2E;
  --panel:          rgba(11, 22, 34, 0.85);
  --panel-border:   rgba(91, 168, 154, 0.15);

  /* Primary — biotexturas teal (from logo) */
  --accent:         #5BA89A;
  --accent-light:   #7EC4B8;
  --accent-soft:    rgba(91, 168, 154, 0.14);

  /* Secondary — cluster yellow */
  --yellow:         #E8D44D;
  --yellow-soft:    rgba(232, 212, 77, 0.12);

  /* Purple — from cluster visualization */
  --purple:         #6B3FA0;
  --purple-deep:    #3B1F70;

  /* Text */
  --ink:            #E8EDF2;
  --muted:          #8A9BB0;

  /* Semantic */
  --good:           #4CAF50;
  --warn:           #E8D44D;
  --danger:         #E57373;

  /* Effects */
  --shadow:         0 24px 80px rgba(0, 0, 0, 0.35);
  --glow-teal:      0 0 40px rgba(91, 168, 154, 0.15);
}
```

### Background Gradient

```css
body {
  background: #0B1622;
  background-image:
    radial-gradient(ellipse at 20% 50%, rgba(91, 168, 154, 0.12) 0%, transparent 60%),
    radial-gradient(ellipse at 80% 20%, rgba(107, 63, 160, 0.08) 0%, transparent 50%),
    radial-gradient(ellipse at 50% 80%, rgba(232, 212, 77, 0.05) 0%, transparent 40%);
}
```

### Buttons

```css
.btn-primary {
  background: linear-gradient(135deg, var(--accent), var(--accent-light));
}
.btn-primary:hover {
  background: linear-gradient(135deg, var(--accent-light), var(--yellow));
}
```

---

## 4. Copy

### index.html

```html
<title>TerraGenesis — Provenance for Every Observation</title>
<meta name="description" content="Verify biological data provenance from TerraScope microscopes on-chain.">
```

### Header

- Eyebrow: `"TerraGenesis"`
- Tagline: `"Provenance for every observation."`

### Hero

- Kicker: `"Open microscopy meets on-chain provenance"`
- H2: `"Prove your microscopy data is real."`
- Lead: `"Each TerraScope microscope has its own cryptographic identity. Captures are hashed, signed, and verifiable on-chain — no intermediaries, no trust required."`
- CTA primary: `"Verify a capture"` (scrolls to verify section)
- Stat card: `"Registered TerraScopes"` / count

### How It Works (3 inline steps — NOT cards)

1. **Capture** — "TerraScope takes a biological image"
2. **Sign** — "Device hashes and signs the capture"
3. **Verify** — "Anyone checks provenance on-chain"

### Verify Section

- Eyebrow: `"Verify"`
- H2: `"Upload a capture and verify its provenance."`

### Registry Section

- Eyebrow: `"Registry"`
- H2: `"TerraScope microscopes registered on-chain."`

### Register (collapsed, attester-only)

- Button label: `"Register a TerraScope"`
- Only visible when connected wallet is authorized attester

### Footer

```
TerraGenesis · A biotexturas project · Built on HardTrust
Contract: 0x... · Chain: ...
```

---

## 5. Images

- Use `demo-capture/capture.jpg` (real TerraScope microscopy image) as hero background at low opacity or as featured image in the verify section
- biotexturas logo as favicon

---

## 6. Typography

Keep **Space Grotesk** — no change.

---

## Acceptance Criteria

- [ ] Page reduced from 11 sections to ~5 sections
- [ ] Zero "HardTrust" in user-facing UI (only in footer footnote)
- [ ] CSS variables updated to teal/purple/yellow palette
- [ ] Verify section promoted to primary position (after hero)
- [ ] Big verdict card (VERIFIED/NOT VERIFIED) instead of 9 pills
- [ ] Story points section removed
- [ ] Workflow reduced to 3 inline steps
- [ ] Register form collapsed behind attester-only toggle
- [ ] Status grid moved to footer
- [ ] index.html title and meta updated
- [ ] Microscopy image visible in UI
