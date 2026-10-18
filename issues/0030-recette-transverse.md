---
labels: [wayfinder:implement]
parent: 0014-map-implementation-status.md
blocked-by: [0024-dimension-config.md, 0025-dimension-i18n.md, 0026-dimension-pages.md, 0027-dimension-sections.md, 0028-dimension-components.md, 0029-dimension-dependencies.md]
status: open
---

# Recette transverse : couverture des 36 règles et invariants de sortie

## Tranche

La passe de recette demandée par le grilling [0019](0019-grilling-verification-status.md) — elle **clôt** son résolution :

- **Couverture** : chacun des 36 identifiants de règle de [0011](0011-grilling-contrat-sortie.md) est couvert par au moins un test noir (`rule` + gravité + message) ; les trous découverts ici sont comblés (dans leur dimension, en corrigeant le code, pas en affaiblissant la règle).
- **Invariants du contrat de sortie** vérifiés de bout en bout sur fixtures : tri des constats par `location.file` puis règle (indépendant de l'ordre d'exécution des contrôles), règle transverse (aucun arrêt sur fichier-clé absent), `--dimension` répétable + virgules masque l'affichage mais laisse le code de sortie et `summary` globaux, ligne `info: N findings hidden by --dimension` exacte, `--errors-only` retire les `todo` sans changer le code ni les compteurs, `--json` strict (rien sur stdout en cas d'arrêt code 2), `format_version: 1`.
- **Unités** : extraction d'imports, clés de dictionnaire (doublons, trim), scan de routes, entrées de sitemap — des tests unitaires là où la fixture de bout en bout est trop coûteuse, si la résolution de 0019 le retient.

## Critères d'acceptance

- [ ] Une matrice de tests (ou une liste annotée dans la résolution) prouve que les 36 règles ont chacune son test noir.
- [ ] Les invariants de filtre/court-circuit ci-dessus sont chacun tenus par un test.
- [ ] La résolution de 0019 est rédigée (choix de fixtures, niveau de détail, Golden ou assertions) et le ticket passe à `closed`.
- [ ] Toute correction de découverture faite ici reste cohérente avec la spec (aucune règle supprimée ni gravité changée).
