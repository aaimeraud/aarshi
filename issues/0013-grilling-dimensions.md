---
labels: [wayfinder:grilling]
parent: 0005-map-status.md
blocked-by: [0006-grilling-invariants-i18n.md, 0007-grilling-invariants-pages.md, 0008-grilling-invariants-config.md]
status: open
---

# Dimensions canoniques de `status` : liste et identifiants stables

## Question

Le filtre `--dimension <nom>` (ticket « Surface de `status` et absorption de `check` ») et le regroupement du rapport (ticket « Contrat de sortie ») exigent une **liste canonique de dimensions** : un ensemble **disjoint** et **exhaustif**, chaque dimension portant un **identifiant stable** (minuscules, tirets) utilisé à la fois comme valeur de `--dimension` et comme clé de groupe en `--json`.

À décider, une fois les trois catalogues résolus (« Invariants i18n… », « Invariants pages, routes, sections et composants… », « Invariants config/projet et outillage… ») :

- La liste exacte des dimensions et leurs identifiants (par ex. `files`, `routes`, `translations`, `config`, `dependencies`, `components`…).
- L'appartenance de chaque invariant retenu par un catalogue à exactement une dimension (aucun trou, aucun chevauchement).
- L'ordre d'affichage des dimensions dans le rapport humain.

## Resolution

<!-- à remplir à la résolution -->
