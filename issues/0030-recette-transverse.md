---
labels: [wayfinder:implement]
parent: 0014-map-implementation-status.md
blocked-by: [0024-dimension-config.md, 0025-dimension-i18n.md, 0026-dimension-pages.md, 0027-dimension-sections.md, 0028-dimension-components.md, 0029-dimension-dependencies.md]
status: closed
---

# Recette transverse : couverture des 36 règles et invariants de sortie

## Tranche

La passe de recette demandée par le grilling [0019](0019-grilling-verification-status.md) — elle **clôt** son résolution :

- **Couverture** : chacun des 36 identifiants de règle de [0011](0011-grilling-contrat-sortie.md) est couvert par au moins un test noir (`rule` + gravité + message) ; les trous découverts ici sont comblés (dans leur dimension, en corrigeant le code, pas en affaiblissant la règle).
- **Invariants du contrat de sortie** vérifiés de bout en bout sur fixtures : tri des constats par `location.file` puis règle (indépendant de l'ordre d'exécution des contrôles), règle transverse (aucun arrêt sur fichier-clé absent), `--dimension` répétable + virgules masque l'affichage mais laisse le code de sortie et `summary` globaux, ligne `info: N findings hidden by --dimension` exacte, `--errors-only` retire les `todo` sans changer le code ni les compteurs, `--json` strict (rien sur stdout en cas d'arrêt code 2), `format_version: 1`.
- **Unités** : extraction d'imports, clés de dictionnaire (doublons, trim), scan de routes, entrées de sitemap — des tests unitaires là où la fixture de bout en bout est trop coûteuse, si la résolution de 0019 le retient.

## Critères d'acceptance

- [x] Une matrice de tests (ou une liste annotée dans la résolution) prouve que les 36 règles ont chacune son test noir.
- [x] Les invariants de filtre/court-circuit ci-dessus sont chacun tenus par un test.
- [x] La résolution de 0019 est rédigée (choix de fixtures, niveau de détail, Golden ou assertions) et le ticket passe à `closed`.
- [x] Toute correction de découverture faite ici reste cohérente avec la spec (aucune règle supprimée ni gravité changée).

## Resolution

La passe est faite ; la matrice des 36 règles et les choix de [0019](0019-grilling-verification-status.md) vivent dans **sa** résolution.

- **Couverture** : les 36 identifiants de [0011](0011-grilling-contrat-sortie.md) avaient chacun déjà leur test noir (règle + gravité + message) — aucun trou n'a été découvert ; la matrice est recensée dans la résolution de 0019.
- **Invariants** : le tri, le sans-arrêt précoce (projet réduit au seul `aarshi.toml`), les filtres (`--dimension` répétable + virgules, ligne info exacte, `summary` et code intacts ; `--errors-only` sans ligne info) sont tenus par le bloc 0030 de `tests/cli.rs` : `recipe_a_project_carried_to_nothing_still_produces_the_full_report`, `recipe_a_missing_i18n_file_is_reported_once_by_i18n`, `recipe_a_missing_sitemap_is_reported_once_by_config`, `recipe_dimension_repeats_and_lists_keep_the_code_and_the_summary`, `recipe_errors_only_hides_the_todos_without_the_info_line`.
- **Réconciliation du doublon [0025](0025-dimension-i18n.md)** : un constat par objet ; les fichiers `lib/i18n/*` sont rapportés une seule fois par la dimension `i18n` (règle 0013), la disambiguïsation est écrite dans la résolution de 0019.
- **Suite** : les deux mises en attente d'[0028](0028-dimension-components.md) restent respectées (classement par dimension de l'objet fautif, rien sur les composants `custom` non utilisés). Les tests unitaires de primitives ne sont pas retenus : la couverture noire suffit. Les tests anciens qui comptaient le doublon ont été réécrits, aucune règle ni gravité n'a changé.
