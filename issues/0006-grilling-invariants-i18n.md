---
labels: [wayfinder:grilling]
parent: 0005-map-status.md
blocked-by: []
status: closed
assignee: aaimeraud
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

**Catalogue i18n retenu.** La référence pour les clés est le dictionnaire de la **langue par défaut** (`languages[0]`), comme aujourd'hui.

**Config i18n (`lib/i18n/`)** — tout en `error`, comparaisons textuelles (pas de parseur TS) :

- `aarshi.toml` (`languages`) ↔ `lib/i18n/config.ts` (`export const languages`) : cohérence, **ordre compris**.
- `defaultLocale` de `config.ts` = `languages[0]`.
- Table `direction` de `config.ts` : une entrée par langue, et `rtl` correct pour `ar`/`he`/`fa`.
- `lib/i18n/index.ts` : un import et une entrée par langue, `MessageKey` issu de `languages[0]`.

**Dictionnaires (`lib/i18n/dictionaries/<locale>.ts`)** :

| Constat | Gravité |
|---|---|
| Fichier de dictionnaire absent pour une langue de `aarshi.toml` | `error` |
| Clé absente par rapport au dictionnaire par défaut | `error` |
| Clé dupliquée dans un même fichier | `error` |
| Valeur vide (règle ci-dessous) | `todo` |
| Clé en trop (présente dans une langue, absente du défaut) | `todo` |
| Dictionnaire orphelin (fichier pour une langue hors `aarshi.toml`) | `todo` |

**Règle « vide »** : une valeur est vide si, **après `trim()`**, elle est vide — donc `""`, `" "` et `"\n"` comptent. (Aujourd'hui `dictionary::is_empty` teste `""` strictement ; à élargir.)

**Cas rattaché** : « clé référencée par une vue mais absente de tous les dictionnaires » n'est **pas** traité ici. Il rejoint le ticket « Invariants pages, routes, sections et composants », où la clé attendue se déduit du **système de fichiers** (page → `"<page>.title"`, section → `"<page>.<section>.title"`). Pas de parseur TSX en v1.

**Sens des comparaisons de clés** : le dictionnaire par défaut est la référence ; les autres langues sont comparées à lui (manquantes → `error`, en trop → `todo`). Aucune comparaison des langues entre elles.
