---
labels: [wayfinder:grilling]
parent: 0014-map-implementation-status.md
blocked-by:
  - 0016-grilling-moteur-status.md
  - 0017-grilling-rendu-json.md
  - 0018-grilling-codes-sortie-alias-check.md
  - 0019-grilling-verification-status.md
status: open
---

# Découpage des livraisons

## Question

Comment livrer `status` en tranches cohérentes ?

- **Séquençage.** Le rapport expose toujours les six dimensions : le moteur doit donc exister avant que les dimensions ne se remplissent. Quelle séquence de commits : moteur + un socle vide, puis une dimension par tranche (config, i18n, pages, sections, components, dependencies) ? Faut-il d'abord l'amont config (ticket « Périmètre »), ou le traiter à part ?
- **Cohérence de chaque tranche.** Que doit contenir une tranche pour être relisible et testable seule (contrôles + tests + libellés de sa dimension) ?
- **PR.** Un seul PR pour toute la carte, ou une pile (amont → moteur → dimensions → doc) ? Rappel : GitButler gère les branches empilées et `but pr`.
- **Ordre vis-à-vis de la doc.** La doc EN + FR est mise à jour avec l'implémentation ([ticket 0012](0012-grilling-livraison-spec.md)) : une tranche finale dédiée, ou chaque tranche met à jour sa part ?

## Resolution
