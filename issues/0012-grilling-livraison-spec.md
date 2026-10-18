---
labels: [wayfinder:grilling]
parent: 0005-map-status.md
blocked-by: [0010-grilling-surface-status.md, 0011-grilling-contrat-sortie.md]
status: closed
assignee: aaimeraud
---

# Livraison et emplacement de la spec de `status`

## Question

Sous quelle forme et à quel emplacement la spec de `status` est-elle livrée, pour qu'une carte d'implémentation (du type `issues/0004-map-implementation.md`) puisse la consommer ?

- Emplacement : section de `README.md` / `README.fr.md` (+ `CONVENTIONS.md` / `CONVENTIONS.fr.md`), résolutions de cette carte (`issues/0005-map-status.md`), ou document dédié ?
- Niveau de détail : la spec inclut-elle le texte exact des messages et le schéma `--json`, ou s'en tient-elle aux règles et invariants ?
- Mise à jour de la doc CLI (liste des commandes, `check` devenu alias) : maintenant ou avec l'implémentation ?
- La carte d'implémentation est-elle un nouveau fichier `issues/` (dont la destination référence cette spec), et faut-il lister explicitement les décisions héritées ?

## Resolution

**La spec vit dans les résolutions de cette carte.** `issues/0005-map-status.md` et les tickets `0006`–`0013` forment la spec : la section « Decisions so far » en est l'index, et chaque ligne renvoie à la résolution qui porte le détail (catalogue des invariants, six dimensions, 36 identifiants de règle, contrat de sortie, schéma `--json`). Aucun document dédié n'est créé : une décision n'existe qu'à un seul endroit.

**Niveau de détail** : la spec fixe les invariants, les gravités, les identifiants de règle, la forme du rapport, les codes de sortie et le schéma `--json`. Le **libellé anglais de chaque `summary`** n'en fait pas partie : c'est du texte de sortie, écrit à l'implémentation et figé par les tests.

**Doc CLI** : mise à jour **avec l'implémentation**, pas avant.

- `README.md` / `README.fr.md` : section `status` (rôle, flags `--dimension` et `--errors-only`, codes de sortie, forme de la sortie humaine, schéma `--json`, mention de `check` comme alias visible).
- `CONVENTIONS.md` / `CONVENTIONS.fr.md` : le catalogue des invariants du projet généré, par dimension — c'est ce que `status` vérifie.
- Tant que `status` n'est pas implémenté, la doc décrit `check` tel quel ; `check` reste fonctionnel en attendant.

**Carte d'implémentation** : un **nouveau fichier** `issues/` (prochain numéro libre, `0014` à ce jour), du type `issues/0004-map-implementation.md`, dont la destination référence cette carte et ses tickets fermés. Elle sera **chartée dans un effort séparé** (session dédiée, comme `0001-map.md` → `0004-map-implementation.md`). Ses décisions héritées **renvoient** aux tickets fermés (liens) au lieu de les recopier : une seule source par décision.
