---
labels: [wayfinder:research]
parent: 0005-map-status.md
blocked-by: []
status: closed
---

# Recherche : comment les CLI conçoivent une commande de diagnostic (`doctor`/`status`)

## Question

Comment des CLI comparables conçoivent-ils une commande de diagnostic (`doctor`, `status`, `check`, `verify`) ? Précisément :

- Quelles **catégories de vérifications** reviennent (fichiers attendus, liens croisés, config, dépendances, versions, état de service) ?
- Quelle **distinction de gravité** est faite (erreur bloquante vs avertissement / reste-à-faire) et comment le **code de sortie** en découle-t-il ?
- Quelle **sortie machine** est proposée et sous quelle forme (`--json` maison et son schéma, JSON-lines, SARIF, autre) ?

Sources primaires à privilégier : documentation et code de CLI établis (par ex. `cargo`, `gh`, `prisma`, `eslint`, `brew doctor`, `flutter doctor`, `npm doctor`, `rustup`), et la spécification SARIF si elle est citée.

Cette recherche éclaire la surface de la commande (ticket « Surface de `status` et absorption de `check` ») et le contrat de sortie (ticket « Contrat de sortie »).

## Resolution

**Réponse** — fichier de recherche : `research/doctor-cli-conventions.md`, conservé avec cette carte : il arrive sur la branche de la carte (et donc sur `dev`) en même temps que la spec. Chaque affirmation y est citée à sa source primaire (docs et code de `npm doctor`, ESLint, `terraform validate`, `brew doctor`, Flutter, `rustup`, GitHub CLI, ShellCheck, Prisma, Cargo ; schéma OASIS SARIF 2.1.0).

Synthèse retenue :

- **Catégories nommées et stables** (à l'image des `groups` de `npm doctor` et des checks énumérables de `brew doctor`) : présence des fichiers attendus, câblage croisé, config `aarshi.toml`, dictionnaires i18n, routes `app/[locale]/`.
- **Gravité** : `error` / `warning` est la convention dominante ; deux niveaux suffisent pour `aarshi status` (`error`, `todo`).
- **Code de sortie** : binaire, **pas** proportionnel au nombre de problèmes ; non nul si et seulement s'il y a au moins une `error`. Réserver un **code dédié à l'erreur interne/config** (à la manière d'ESLint `2` et ShellCheck `2`), distinct de `1` (diagnostic en erreur) et `0` (succès).
- **Sortie machine** : `--json` maison avec **schéma versionné** (`format_version`, mineur rétrocompatible) et diagnostics structurés `{ id/ruleId, severity, summary, detail, location }`, plus des compteurs — forme de `terraform validate -json` et du formatter `json` d'ESLint. **SARIF n'est pas requis en v1** (aucun outil comparable cité ne l'embarque par défaut).
- **JSON strict sur stdout** en mode `--json` ; le mode humain seul imprime l'en-tête décoré (`AARSHI_NO_HEADER`).
- **Identifiant de règle stable** par constat (`missing-dictionary`, `unwired-section`, …) pour les filtres, la déduplication et la stabilité des tests.
- **Piège à éviter** (cf. `gh auth status`) : un JSON valide avec un code `0` alors que des `error` existent.

Éclaire les tickets « Surface de `status` et absorption de `check` » et « Contrat de sortie : rendu humain, `--json`, codes de sortie ».
