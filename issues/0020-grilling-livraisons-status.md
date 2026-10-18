---
labels: [wayfinder:grilling]
parent: 0014-map-implementation-status.md
blocked-by:
  - 0016-grilling-moteur-status.md
  - 0017-grilling-rendu-json.md
  - 0018-grilling-codes-sortie-alias-check.md
  - 0019-grilling-verification-status.md
status: closed
---

# Découpage des livraisons

## Question

Comment livrer `status` en tranches cohérentes ?

- **Séquençage.** Le rapport expose toujours les six dimensions : le moteur doit donc exister avant que les dimensions ne se remplissent. Quelle séquence de commits : moteur + un socle vide, puis une dimension par tranche (config, i18n, pages, sections, components, dependencies) ? Faut-il d'abord l'amont config (ticket « Périmètre »), ou le traiter à part ?
- **Cohérence de chaque tranche.** Que doit contenir une tranche pour être relisible et testable seule (contrôles + tests + libellés de sa dimension) ?
- **PR.** Un seul PR pour toute la carte, ou une pile (amont → moteur → dimensions → doc) ? Rappel : GitButler gère les branches empilées et `but pr`.
- **Ordre vis-à-vis de la doc.** La doc EN + FR est mise à jour avec l'implémentation ([ticket 0012](0012-grilling-livraison-spec.md)) : une tranche finale dédiée, ou chaque tranche met à jour sa part ?

## Resolution

**Séquençage retenu (graph de tickets [wayfinder:implement] 0022–0031)** : l'amont config d'abord ([0022](0022-amont-options-config.md), en parallèle du socle) ; **socle/tracer** [0023](0023-socle-tracer-i18n.md) (surface + moteur + sortie + quatre règles i18n, pas un socle vide) ; puis une **dimension par tranche** — [0024 config](0024-dimension-config.md), [0025 i18n](0025-dimension-i18n.md), [0026 pages + primitives partagées](0026-dimension-pages.md), [0027 sections](0027-dimension-sections.md), [0028 components](0028-dimension-components.md), [0029 dependencies](0029-dimension-dependencies.md) — exécutées en parallèle dès que leurs arêtes de blocage (primitives de 0026, options de 0022) sont levées ; **recette transverse** [0030](0030-recette-transverse.md) (matrice des 36 règles, invariants de sortie) ; **doc finale** [0031](0031-doc-status.md).

**Cohérence de chaque tranche** : code + tests noirs de sa dimension + libellés conformes aux conventions de wording de [0017](0017-grilling-rendu-json.md), dont la sortie est figée par ses tests ; le ticket écrit la résolution des grillings qu'il porte et passe à `closed`. Chaque tranche est intégrée individuellement sur la branche d'implémentation, suite de tests verte après chaque intégration.

**PR** : pas ouvert pendant l'exécution (le tracker est local). La livraison finale est la branche d'implémentation ; l'ouverture d'un PR (simple ou pile GitButler via `but pr`) reste à la demande, au moment où la demande en aura.

**Doc** : une **tranche finale dédiée** ([0031](0031-doc-status.md)) après la recette, conformément à l'accord « avec l'implémentation, pas avant » de [0012](0012-grilling-livraison-spec.md) : le libellé étant figé par les tests, une seule passe rédige README + CONVENTIONS (EN + FR) en parité qu'une mise à jour à chaque tranche ferait bouger en continu.
