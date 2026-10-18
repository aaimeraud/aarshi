---
labels: [wayfinder:grilling]
parent: 0014-map-implementation-status.md
blocked-by: [0017-grilling-rendu-json.md]
status: open
---

# Documentation de `status` (README et CONVENTIONS, EN + FR)

## Question

Le [ticket de livraison de la spec](0012-grilling-livraison-spec.md) fixe que la doc est mise à jour **avec** l'implémentation, pas avant. Quel contenu exact, et dans quels fichiers ?

- `README.md` / `README.fr.md` : section `status` — rôle, flags `--dimension <nom>` (valeurs = les six identifiants) et `--errors-only`, codes de sortie `0`/`1`/`2`, forme de la sortie humaine (rapport par dimension + verdict), schéma `--json` (ou renvoi), mention de `check` comme alias visible. La ligne `check` actuelle (ligne 20) est remplacée.
- `CONVENTIONS.md` / `CONVENTIONS.fr.md` : catalogue des invariants du projet généré, par dimension — c'est ce que `status` vérifie. Les 36 règles sont-elles listées une par une, ou regroupées par dimension avec les gravités ?
- **En-tête du README** : la ligne `Status: the v1 CLI is in review (PR #4)…` (ligne 5) doit-elle être mise à jour, et vers quoi ?
- **Parité EN/FR** : les deux versions portent-elles exactement le même contenu ?

## Resolution
