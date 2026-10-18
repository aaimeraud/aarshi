---
labels: [wayfinder:implement]
parent: 0014-map-implementation-status.md
blocked-by: [0023-socle-tracer-i18n.md]
status: closed
---

# Dimension `i18n` complète : config.ts, index.ts, doublons, excédents, orphelins

## Tranche

Compléter la dimension `i18n` au catalogue [0006](0006-grilling-invariants-i18n.md) — les quatre règles du tracer ([0023](0023-socle-tracer-i18n.md)) existent déjà. Restent :

- `i18n.default-locale-mismatch` : `defaultLocale` de `lib/i18n/config.ts` ≠ `languages[0]` (`error`).
- `i18n.direction-mismatch` : table `direction` incomplète ou incorrecte — une entrée par langue, `rtl` correct pour `ar`/`he`/`fa` (`error`).
- `i18n.index-mismatch` : `lib/i18n/index.ts` désynchronisé — un import et une entrée par langue, `MessageKey` issu de `languages[0]` (`error`).
- `i18n.key-duplicate` : clé dupliquée dans un même dictionnaire (`error`).
- `i18n.key-extra` : clé présente dans une langue, absente du dictionnaire de référence (`todo`).
- `i18n.dictionary-orphan` : dictionnaire pour une langue hors `aarshi.toml` (`todo`).

Rappels de spec : comparaisons **textuelles** (pas de parseur TS) ; la langue par défaut est l'unique référence (aucune comparaison entre les autres langues) ; les clés attendues des vues/sections relèvent des dimensions `pages`/`sections`, pas de celle-ci.

## Critères d'acceptance

- [x] Chaque règle a au moins un test noir (constat déclenché sur fixture, `rule`, gravité, message).
- [x] Un dictionnaire avec doublon, valeurs vides, clés en trop et orphelin mélangeant les gravités produit des constats triés par fichier puis règle, et le verdict final compte correctement.
- [x] `config.ts` absent ou illisible produit un `error` (`i18n.languages-mismatch` ou file-missing selon le libellé retenu) sans arrêter les autres dimensions. Libellé retenu : `i18n.languages-mismatch` (« is missing or unreadable »), la dimension `config` gardant `config.file-missing` pour son ticket.
