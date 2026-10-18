---
labels: [wayfinder:implement]
parent: 0014-map-implementation-status.md
blocked-by: [0030-recette-transverse.md]
status: open
---

# Documentation de `status` (README et CONVENTIONS, EN + FR)

## Tranche

La doc EN + FR mise à jour **avec** l'implémentation [décision 0012](0012-grilling-livraison-spec.md) ; cette tranche **clôt** le grilling [0021](0021-grilling-doc-status.md) qui fixe le contenu exact :

- `README.md` / `README.fr.md` : section `status` — rôle, flags `--dimension <nom>` (valeurs = les six identifiants) et `--errors-only`, codes de sortie `0`/`1`/`2`, forme de la sortie humaine (rapport par dimension + verdict), schéma `--json` (ou renvoi), mention de `check` comme alias visible ; la ligne `aarshi check` actuelle (ligne 20 EN) est remplacée ; la ligne `Status:` du README (v1 in review, PR #4) est mise à jour.
- `CONVENTIONS.md` / `CONVENTIONS.fr.md` : catalogue des invariants du projet généré, **par dimension** (les 36 règles, regroupées avec leurs gravités — le niveau de détail est tranché ici).
- **Parité EN/FR** : les deux portent exactement le même contenu.

## Critères d'acceptance

- [ ] Une commande `status` documentée avec ses trois mécanismes de sortie (humain, codes, `--json`).
- [ ] Le catalogue des invariants par dimension est présent dans CONVENTIONS (EN + FR), avec les 36 identifiants de règle et leurs gravités.
- [ ] La mention `check` surviv sous forme d'alias visible ; aucune doc ne décrit `Command::Check`.
- [ ] Les paires README/README.fr et CONVENTIONS/CONVENTIONS.fr sont redistribuées symétriquement (contenu identique, langue seule différente).
