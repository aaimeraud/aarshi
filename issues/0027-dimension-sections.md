---
labels: [wayfinder:implement]
parent: 0014-map-implementation-status.md
blocked-by: [0026-dimension-pages.md]
status: open
---

# Dimension `sections` : imports pendants, clés, marqueurs, orphelines

## Tranche

Remplir la dimension `sections` (catalogue [0007](0007-grilling-invariants-pages.md)) sur la base des primitives de [0026](0026-dimension-pages.md) :

| Règle | Gravité | Constat |
|---|---|---|
| `sections.file-missing` | `error` | import pendant (vue → `sections/<section>.tsx` absent) |
| `sections.title-key-missing` | `error` | clé `"<page>.<section>.title"` absente d'un dictionnaire |
| `sections.marker-missing` | `error` | `// aarshi:imports:end` ou `{/* aarshi:sections:end */}` absent de la vue |
| `sections.orphan` | `todo` | section jamais importée |
| `sections.import-broken` | `error` | import cassé dans une section |

Rappel de spec : l'attente des clés section vient du système de fichiers (`views/<page>/sections/<section>.tsx` → `"<page>.<section>.title"`) ; les marqueurs absents rendent `add -s` inopérant mais ne sont pas détectés par `check` aujourd'hui.

## Critères d'acceptance

- [ ] Chaque règle a au moins un test noir (constat déclenché sur fixture, `rule`, gravité, message).
- [ ] Une vue dont un seul des deux marqueurs manque produit un constat ; une vue complète n'en produit pas.
- [ ] Une section orpheline (`todo`) compte dans le verdict `✓ project is conformant (N to do)` sans faire sortir du code `0`.
