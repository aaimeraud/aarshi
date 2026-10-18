---
labels: [wayfinder:grilling]
parent: 0014-map-implementation-status.md
blocked-by: [0016-grilling-moteur-status.md, 0017-grilling-rendu-json.md]
status: open
---

# Vérification : fixtures, couverture des 36 règles, tests `--json`

## Question

Comment le comportement de `status` est-il tenu par les tests ?

- **Fixtures.** Comment fabriquer les « projets cassés » : partir d'un projet généré par `setup` puis le muter (route manquante, clé supprimée, import cassé, lockfile incohérent…), ou écrire une fabrique dédiée ? Où placer les helpers : `tests/cli.rs` ou fichiers séparés (par dimension) ? Faut-il une variante de `Sandbox` qui produit un projet minimal sans passer par `setup` ?
- **Couverture.** Les 36 règles sont-elles couvertes une par une par un test noir, ou seulement les cas structurants (les six dimensions, le cas propre, plusieurs `error` et `todo` mêlés) ? À quel niveau de détail : message exact, identifiant de règle, ordre, code de sortie.
- **Contrat `--json`.** Comment figer le schéma : comparaison structurée du document, golden file, ou assertions ciblées par champ ? Où vérifier `format_version`, `summary` global sous `--dimension`/`--errors-only`, et l'absence de sortie sur stdout en cas d'arrêt.
- **Unités.** Faut-il des tests unitaires sur les primitives de lecture du moteur (extraction d'imports, clés, sitemap) en plus des tests noirs ?

## Resolution
