---
labels: [wayfinder:research]
parent: 0005-map-status.md
blocked-by: []
status: open
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

<!-- à remplir à la résolution : synthèse + lien vers le fichier de recherche sur la branche research/… -->
