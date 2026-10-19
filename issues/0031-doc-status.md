---
labels: [wayfinder:implement]
parent: 0014-map-implementation-status.md
blocked-by: [0030-recette-transverse.md]
status: closed
---

# Documentation de `status` (README et CONVENTIONS, EN + FR)

## Tranche

La doc EN + FR mise à jour **avec** l'implémentation [décision 0012](0012-grilling-livraison-spec.md) ; cette tranche **clôt** le grilling [0021](0021-grilling-doc-status.md) qui fixe le contenu exact :

- `README.md` / `README.fr.md` : section `status` — rôle, flags `--dimension <nom>` (valeurs = les six identifiants) et `--errors-only`, codes de sortie `0`/`1`/`2`, forme de la sortie humaine (rapport par dimension + verdict), schéma `--json` (ou renvoi), mention de `check` comme alias visible ; la ligne `aarshi check` actuelle (ligne 20 EN) est remplacée ; la ligne `Status:` du README (v1 in review, PR #4) est mise à jour.
- `CONVENTIONS.md` / `CONVENTIONS.fr.md` : catalogue des invariants du projet généré, **par dimension** (les 36 règles, regroupées avec leurs gravités — le niveau de détail est tranché ici).
- **Parité EN/FR** : les deux portent exactement le même contenu.

## Critères d'acceptance

- [x] Une commande `status` documentée avec ses trois mécanismes de sortie (humain, codes, `--json`).
- [x] Le catalogue des invariants par dimension est présent dans CONVENTIONS (EN + FR), avec les 36 identifiants de règle et leurs gravités.
- [x] La mention `check` surviv sous forme d'alias visible ; aucune doc ne décrit `Command::Check`.
- [x] Les paires README/README.fr et CONVENTIONS/CONVENTIONS.fr sont redistribuées symétriquement (contenu identique, langue seule différente).

## Resolution

Fait, avec les choix de contenu de la résolution de [0021](0021-grilling-doc-status.md).

- **README** : la puce `aarshi check` est remplacée par la section `aarshi status` (rôle, sortie humaine, codes `0`/`1`/`2`, `--dimension` et `--errors-only`, `--json` et `format_version: 1`, alias `check` visible), avec renvoi vers le catalogue de CONVENTIONS. La ligne `Status:` du README est étendue à « `aarshi status` is now implemented » (miroir FR exact).
- **CONVENTIONS** : nouvelle sous-section « Invariants of the generated project » dans la section Generated projects — les 36 règles de [0011](0011-grilling-contrat-sortie.md) groupées par dimension (config, i18n, pages, sections, components, dependencies), un tiret par règle (identifiant + gravité + objet), et per-rule vs per-dimension tranché : par dimension.
- **Parité** : les quatre fichiers sont livrés en paires symétriques EN/FR, même contenu et même ordre.
