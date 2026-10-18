---
labels: [wayfinder:grilling]
parent: 0014-map-implementation-status.md
blocked-by: [0016-grilling-moteur-status.md]
status: open
---

# Rendu humain, `--json` et wording des messages

## Question

Le [contrat de sortie](0011-grilling-contrat-sortie.md) fixe la forme ; ce ticket fixe sa **mise en œuvre** :

- **Rendu humain.** Une ligne par dimension dans l'ordre canonique, `✓` vert quand la dimension est propre, constats indentés préfixés `error:` (rouge gras) ou `todo:` (jaune gras), sur `src/output.rs`. Ajout du helper `todo()` symétrique de `warning()`. Cas « rien à signaler » et ligne de verdict (trois formes, mise au pluriel). Ligne `info: N findings hidden by --dimension` quand le filtre masque un constat. `print_header()` reste affiché en mode humain (`AARSHI_NO_HEADER`).
- **`--json`.** Structs de sérialisation (`format_version`, `summary`, `dimensions[]`, `findings[]`), ordre des clés, champs optionnels omis quand absents, compteurs par dimension. JSON strict sur stdout, jamais de bannière ni de couleur ; rien sur stdout en cas d'arrêt. Ajout de `serde_json` à `Cargo.toml`.
- **Wording.** Convention de rédaction des 36 phrases `summary` en anglais (et du champ `detail` : valeur attendue, cause), avec la grammaire des `location`. La spec laisse le texte à l'implémentation : il est écrit ici et figé par les tests. Produit-on la liste complète des libellés, ou seulement les conventions de rédaction ?

## Resolution
