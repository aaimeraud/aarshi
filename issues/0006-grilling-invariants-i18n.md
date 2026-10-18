---
labels: [wayfinder:grilling]
parent: 0005-map-status.md
blocked-by: []
status: open
---

# Invariants i18n : lesquels `status` vérifie-t-il, et à quelle gravité ?

## Question

Parmi les invariants i18n/dictionnaires candidats ci-dessous, lesquels `status` vérifie-t-il en v1, avec quelle règle exacte, et avec quel classement `error` (conformité cassée) / `todo` (reste-à-faire attendu) ?

Candidats relevés dans `src/check.rs`, `src/dictionary.rs`, `src/templates.rs`, `src/lang.rs` :

**Config i18n (`lib/i18n/`)** — partiellement couvert par `check` aujourd'hui :

- `aarshi.toml` (`languages`) ↔ `lib/i18n/config.ts` (`export const languages`) : cohérence, ordre compris.
- `defaultLocale` de `config.ts` = `languages[0]` (aujourd'hui **non vérifié**).
- Table `direction` de `config.ts` complète et correcte (`rtl` pour `ar`/`he`/`fa`).
- `lib/i18n/index.ts` : imports, map `{ fr, en, … }` et `MessageKey` cohérents avec `config.ts` (aujourd'hui **non vérifié**).

**Dictionnaires (`lib/i18n/dictionaries/<locale>.ts`)** :

- Fichier présent pour chaque langue de `aarshi.toml`.
- Valeur vide pour une clé (`""`), par langue — `check` la reporte ; un blanc (`" "`) compte-t-il comme vide ?
- Clé absente par rapport au dictionnaire de la langue par défaut.
- Clé en trop (présente dans une langue, absente du défaut).
- Clé dupliquée dans un même dictionnaire.
- Clé référencée par une vue mais absente de **tous** les dictionnaires (aujourd'hui invisible : `check` se base sur le dictionnaire par défaut, donc une clé manquante partout n'est jamais signalée).

## Resolution

<!-- à remplir à la résolution : invariants retenus, règle exacte, gravité error/todo, exemples -->
