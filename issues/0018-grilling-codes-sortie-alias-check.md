---
labels: [wayfinder:grilling]
parent: 0014-map-implementation-status.md
blocked-by: [0015-grilling-perimetre-implementation.md]
status: open
---

# Codes de sortie et absorption de `check`

## Question

- **Plomberie des codes.** `main()` (`src/main.rs`) mappe aujourd'hui toute erreur sur le code `1` et imprime `error:`. Comment distinguer `0` (aucune `error`), `1` (au moins une `error`) et `2` (`status` ne peut pas analyser : `aarshi.toml` absent, illisible ou invalide, message sur stderr, rien sur stdout) ? Type d'erreur dédié, code de sortie renvoyé par `status::run`, ou autre. Quel effet sur les autres sous-commandes, qui gardent leur comportement actuel ?
- **Alias `check`.** `#[command(visible_alias = "check")]` sur `status`, suppression de `Command::Check`, `about` (`Report what is missing or wrong in the Next.js project`) et place de `status` en dernier dans l'aide (`setup`, `add`, `add-lang`, `icons`, `status`).
- **Message hors projet.** `error: no aarshi.toml in this folder; nothing to check` sur stderr, code `2` — il remplace le message actuel de `check` (`not an aarshi project: …`) et celui des autres commandes reste-t-il inchangé ?
- **Migration des tests.** Le test `check_reports_empty_translations_until_they_are_filled` change de comportement : la traduction vide devient un `todo`, donc `status` sort `0` avec un rapport différent. Quels tests réécrire, lesquels ajouter, et faut-il un test dédié par code de sortie ?

## Resolution
