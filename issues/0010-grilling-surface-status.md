---
labels: [wayfinder:grilling]
parent: 0005-map-status.md
blocked-by: [0009-research-doctor-cli.md]
status: open
assignee: aaimeraud
---

# Surface de `status` et absorption de `check`

## Question

**Direction retenue au charting** : `status` absorbe `check` ; `check` devient un alias. Restent à trancher les détails :

- Mécanique de l'alias : `check` reste une sous-commande `clap` qui délègue à `status` ? Affiche-t-elle un message (`warning:` de dépréciation) ou reste-t-elle silencieuse ? Les deux commandes partagent-elles exactement les mêmes flags (y compris `--json`) ?
- Comportement hors projet aarshi (pas d'`aarshi.toml`) : même message et même code de sortie que `check` aujourd'hui (`bail!("not an aarshi project …")`) ?
- Flags non liés au format : `status` accepte-t-il un filtre par dimension (`--dimension <nom>`), un mode court (`--short`), un mode qui n'affiche que les erreurs (`--errors-only`) ? Quels défauts ?
- Place de `status` dans l'aide `clap` (ordre des sous-commandes) et dans la documentation (`README.md` / `CONVENTIONS.md` et leurs versions FR).

Les flags liés au format (`--json`) et le détail du rendu appartiennent au ticket « Contrat de sortie ».

S'appuie sur [Recherche : comment les CLI conçoivent une commande de diagnostic (`doctor`/`status`)](0009-research-doctor-cli.md) : catégories nommées et ciblables par argument nommé (comme `brew doctor <check>`).

## Resolution

<!-- à remplir à la résolution -->
