---
labels: [wayfinder:grilling]
parent: 0005-map-status.md
blocked-by: [0006-grilling-invariants-i18n.md, 0007-grilling-invariants-pages.md, 0008-grilling-invariants-config.md]
status: closed
assignee: aaimeraud
---

# Dimensions canoniques de `status` : liste et identifiants stables

## Question

Le filtre `--dimension <nom>` (ticket « Surface de `status` et absorption de `check` ») et le regroupement du rapport (ticket « Contrat de sortie ») exigent une **liste canonique de dimensions** : un ensemble **disjoint** et **exhaustif**, chaque dimension portant un **identifiant stable** (minuscules, tirets) utilisé à la fois comme valeur de `--dimension` et comme clé de groupe en `--json`.

À décider, une fois les trois catalogues résolus (« Invariants i18n… », « Invariants pages, routes, sections et composants… », « Invariants config/projet et outillage… ») :

- La liste exacte des dimensions et leurs identifiants (par ex. `files`, `routes`, `translations`, `config`, `dependencies`, `components`…).
- L'appartenance de chaque invariant retenu par un catalogue à exactement une dimension (aucun trou, aucun chevauchement).
- L'ordre d'affichage des dimensions dans le rapport humain.

## Resolution

**Six dimensions**, dans cet ordre (fondations → feuilles). L'identifiant est en minuscules, sans espace, et sert à la fois de valeur à `--dimension` et de clé de groupe en `--json`.

| # | Identifiant | Contenu |
|---|---|---|
| 1 | `config` | `aarshi.toml` (présence, validité stricte, clé manquante), options persistées (`cache_components`, `site_url`, `ui`, `icons`), redirect de `next.config.ts`, fichiers et dossiers de base, marqueurs de fichiers, imports cassés des fichiers de base (`app/[locale]/layout.tsx`) |
| 2 | `i18n` | `lib/i18n/config.ts` (`languages` ↔ `aarshi.toml`, `defaultLocale`, `direction`), `lib/i18n/index.ts`, dictionnaires (fichier absent, clé manquante / dupliquée / vide / en trop, dictionnaire orphelin) |
| 3 | `pages` | vues ↔ routes (deux sens), sitemap ↔ page, clé `"<page>.title"`, import de vue, imports cassés des vues et des routes |
| 4 | `sections` | import pendant, clé `"<page>.<section>.title"`, marqueurs de vue, section orpheline, imports cassés des sections |
| 5 | `components` | icônes `.svg` ↔ `.tsx`, imports cassés dans `components/` |
| 6 | `dependencies` | dépendances déclarées, imports de paquets npm non déclarés, lockfile ↔ `package_manager`, `node_modules` |

**Règle d'appartenance** : un constat appartient à **exactement une** dimension, celle de l'**objet qu'il concerne** — jamais celle du fichier qui l'a détecté. Un import cassé va dans la dimension de l'objet fautif (vue ou route → `pages`, section → `sections`, fichier de base → `config`, composant → `components`) ; un **import de paquet npm non déclaré** va dans `dependencies`, parce que l'objet concerné est la dépendance, pas le fichier. Pas de dimension `links`.

**Propriété à tenir** : la liste est **disjointe et exhaustive** vis-à-vis des trois catalogues — chaque invariant retenu par « Invariants i18n… », « Invariants pages, routes, sections et composants… » et « Invariants config/projet et outillage… » a sa place, et une seule.

**Conséquences** : `--dimension <nom>` accepte ces six identifiants (validation `clap`) ; `--json` les utilise comme clés de groupe ; l'ordre ci-dessus est celui du rapport humain.
