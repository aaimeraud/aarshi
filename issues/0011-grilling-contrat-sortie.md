---
labels: [wayfinder:grilling]
parent: 0005-map-status.md
blocked-by: [0006-grilling-invariants-i18n.md, 0007-grilling-invariants-pages.md, 0008-grilling-invariants-config.md]
status: open
---

# Contrat de sortie : rendu humain, `--json`, codes de sortie

## Question

Une fois le catalogue d'invariants fixé (tickets « catalogue »), quel est le contrat de sortie exact de `status` ?

**Rendu humain** :

- Structure exacte du rapport groupé par dimension (titre de dimension, items, statut par dimension), en s'appuyant sur `src/output.rs` (`error`, `warning`, `success`, `info`).
- Ligne de résumé / verdict final (par ex. « projet conforme » vs « N erreurs, M à faire »), et le cas « rien à signaler ».
- Affichage des invariants **OK** : listés ou seulement comptés ?

**Codes de sortie** :

- Confirmer : non-zéro si et seulement s'il y a au moins une `error` ; des `todo` seuls → code 0. Quel code exact pour l'échec (1 ?) ?

**Sortie machine `--json`** :

- Schéma exact : champs par dimension et par item (gravité, identifiant/famille de règle, `file`, `line` ?, clé de dictionnaire, locale, message), présence d'un champ de version de schéma et d'un résumé global.
- Stabilité : le schéma est-il un contrat public (à documenter et versionner) ou un détail interne ?
- Interaction avec `print_header` (`AARSHI_NO_HEADER`), les couleurs et le TTY en mode `--json` (JSON strict sur stdout ?).
- Impact dépendance : introduire `serde_json` dans `Cargo.toml`.

## Resolution

<!-- à remplir à la résolution -->
