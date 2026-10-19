---
labels: [wayfinder:grilling]
parent: 0005-map-status.md
blocked-by: [0009-research-doctor-cli.md]
status: closed
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

**Terme canonique** : **dimension**. C'est le mot de la carte et de cette spec ; la recherche disait « catégorie », `npm doctor` « group », `brew doctor` « check ». « Dimension » sert de vocabulaire et de nom de flag. (Où le consigner relève du ticket « Livraison et emplacement de la spec » ; les valeurs et identifiants sont produits par le nouveau ticket « Dimensions canoniques de `status` ».)

**Alias `check`** : `status` porte `#[command(visible_alias = "check")]`. `aarshi check` continue de fonctionner à l'identique (mêmes flags, même sortie, même code), et l'aide liste `check` comme alias de `status`. Aucun message de dépréciation (le CLI n'est pas publié). La sous-commande `Command::Check` disparaît.

**Filtre par dimension** : option `--dimension <nom>`, répétable, virgules acceptées (`--dimension routes,translations`), défaut = toutes les dimensions. Les valeurs valides forment une liste fermée validée par `clap` (`value_parser`) : une dimension inconnue est une erreur d'usage (code 2, comme `clap` le fait déjà, ce qui s'aligne sur le code « usage/interne » de la recherche). La **liste canonique des valeurs** est produite par le nouveau ticket « Dimensions canoniques de `status` ».

**Modes courts** : `--errors-only` **uniquement** — masque les `todo`, garde les `error` et un résumé ; le code de sortie reste inchangé (les `todo` ne l'affectent jamais). Pas de `--short` : c'est de la mise en forme fragmentaire, sans valeur pour l'automatisation (couverte par `--json`) et coûteuse à figer dans le contrat de sortie humaine.

**Hors projet aarshi** : arrêt immédiat, sans rapport dégradé, avec `error: no aarshi.toml in this folder; nothing to check` sur stderr. Ce message remplace celui de `check` (`not an aarshi project: …`). Le code de sortie associé est tranché par le ticket « Contrat de sortie ».

**Place dans l'aide** : `status` est déclaré en dernier, à la place de `check` (`setup`, `add`, `add-lang`, `icons`, `status`).

**Doc** : la mise à jour de `README.md` / `README.fr.md` et `CONVENTIONS.md` / `CONVENTIONS.fr.md` est déléguée au ticket « Livraison et emplacement de la spec de `status` ».

**Wording (niveau spec, ajustable à l'implémentation)** : texte d'aide `about` proposé — `Report what is missing or wrong in the Next.js project`.

**Interactions laissées au ticket « Contrat de sortie »** : l'effet du filtre `--dimension` et de `--errors-only` sur le code de sortie et sur la sortie `--json`.
