---
labels: [wayfinder:map]
status: open
parent: 0001-map.md
---

# Carte : commande `aarshi status`

## Destination

Une spec validée et implémentable de `aarshi status` : une commande de diagnostic **en lecture seule**, lancée dans un projet généré par aarshi (`aarshi.toml` présent), qui indique au dev si le scaffolding est complet et conforme, et ce qu'il lui reste à faire. L'implémentation viendra dans une carte séparée (même schéma que `issues/0001-map.md` → `issues/0004-map-implementation.md`).

## Notes

- Domaine : CLI `aarshi` (Rust, `clap`, `inquire`, `colored`), diagnostic d'un projet Next.js généré (App Router, `views/` + `sections/` + `app/[locale]/` + dictionnaires typés, `aarshi.toml`).
- Hérite de la spec et des décisions de `issues/0001-map.md` et `issues/0004-map-implementation.md`.
- Compétences à consulter : `grilling` + `domain-modeling` par défaut ; `research` pour le ticket de recherche.
- Matière première : un inventaire des invariants du scaffolding a été dressé pendant la session de charting (relevés dans `src/check.rs`, `src/dictionary.rs`, `src/add.rs`, `src/templates.rs`, `src/setup.rs`, `src/config.rs`, `src/project.rs`) ; les candidats sont listés dans le corps des trois tickets « catalogue ».
- **Directions fixées au charting** (elles cadrent les tickets) : `status` absorbe `check`, qui devient un alias ; rapport groupé par dimension ; deux gravités, `error` (conformité cassée) et `todo` (reste-à-faire attendu) ; v1 = présence + câblage, sémantique TS reportée ; attentes dérivées de `aarshi.toml` + découverte de l'état réel, pas de manifeste figé ; le travail manuel du dev est couvert ; lecture seule stricte ; sortie humaine + `--json`.
- Préférences standing : sortie CLI en anglais (cohérente avec `error:` / `success:` / `info:`), issues en français ; aucune écriture dans le projet analysé.

## Decisions so far

<!-- l'index : une ligne par ticket fermé, avec son gist et le lien vers le détail -->

## Not yet specified

- Extensibilité : `status` doit-il permettre au dev de déclarer ses propres contrôles (registre, config) ?
- Prise en compte des commandes aarshi futures dans le catalogue d'invariants (par ex. un futur `aarshi update`).
- Comportement sur un projet généré par une version antérieure d'aarshi (versionnage du scaffolding, migration).

## Out of scope

- Analyse sémantique TypeScript/AST (parser les imports, vérifier qu'une section est réellement rendue) : reportée ; la spec v1 s'en tient à la présence et au câblage. Pourra faire l'objet de sa propre carte.
- `--fix` (réparation automatique) : écarté de cette spec.
- Projets Next.js sans `aarshi.toml` (mode dégradé) : le périmètre est le projet généré par aarshi.
