---
labels: [wayfinder:grilling]
parent: 0014-map-implementation-status.md
blocked-by: [0017-grilling-rendu-json.md]
status: closed
---

# Documentation de `status` (README et CONVENTIONS, EN + FR)

## Question

Le [ticket de livraison de la spec](0012-grilling-livraison-spec.md) fixe que la doc est mise à jour **avec** l'implémentation, pas avant. Quel contenu exact, et dans quels fichiers ?

- `README.md` / `README.fr.md` : section `status` — rôle, flags `--dimension <nom>` (valeurs = les six identifiants) et `--errors-only`, codes de sortie `0`/`1`/`2`, forme de la sortie humaine (rapport par dimension + verdict), schéma `--json` (ou renvoi), mention de `check` comme alias visible. La ligne `check` actuelle (ligne 20) est remplacée.
- `CONVENTIONS.md` / `CONVENTIONS.fr.md` : catalogue des invariants du projet généré, par dimension — c'est ce que `status` vérifie. Les 36 règles sont-elles listées une par une, ou regroupées par dimension avec les gravités ?
- **En-tête du README** : la ligne `Status: the v1 CLI is in review (PR #4)…` (ligne 5) doit-elle être mise à jour, et vers quoi ?
- **Parité EN/FR** : les deux versions portent-elles exactement le même contenu ?

## Resolution

La doc est livrée, en parité stricte avec [0031](0031-doc-status.md).

- **README / README.fr** : la ligne `aarshi check` (dernière puce des Commandes) est remplacée par une section `aarshi status` (sous-titre `###`, toujours en fin de commandes — setup, add, add-lang, icons, status) : rôle (diagnostic en lecture seule du projet généré), les trois mécanismes de sortie (rapport humain une ligne par dimension avec `✓`, constats `error:`/`todo:` indentés et verdict final ; codes de sortie `0`/`1`/`2` avec leur sens ; `--json` avec `format_version: 1`, `summary` global et `findings` `rule`/`severity`/`summary`/`detail`/`location`), les flags `--dimension` (répétable, virgules acceptées, valeurs = les six identifiants, filtrage de l'affichage seulement avec ligne `info: N findings hidden by --dimension`) et `--errors-only`, et la mention de `check` comme alias visible (`aarshi check` produit le même rapport). Aucune doc ne décrit `Command::Check`.
- **En-tête du README** : la ligne `Status:` est mise à jour en « le CLI v1 est en revue (PR #4), et `aarshi status` est désormais implémenté » — choix de compléter la ligne existante plutôt que de la réécrire, les deux versions se répondant mot pour mot.
- **CONVENTIONS / CONVENTIONS.fr** : le catalogue des invariants du projet généré vit dans une sous-section « Invariants du projet généré » de la section « Generated projects ». Les 36 règles de [0011](0011-grilling-contrat-sortie.md) y sont listées une par une (**par dimension**, dans l'ordre canonique config, i18n, pages, sections, components, dependencies) — un tiret par règle avec son identifiant, sa gravité entre parenthèses et son objet, suivi de l'énoncé que `aarshi status` est ce qui les vérifie. Le choix regroupement par dimension plutôt que par gravité ou par fichier : le catalogue découpe les objets comme les dimensions du rapport, un fichier concerné n'est pas un invariant.
- **Parité EN/FR** : mêmes paragraphes, même ordre, même contenu (langue seule différente).
