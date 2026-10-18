---
labels: [wayfinder:grilling]
parent: 0014-map-implementation-status.md
blocked-by: []
status: closed
---

# Périmètre de la carte d'implémentation

## Question

La spec exige que `status` contrôle les options persistées `ui`, `icons`, `cache_components` et `site_url` ([invariants config/outillage](0008-grilling-invariants-config.md)), mais l'écriture de ces clés n'existe pas encore : `Config::to_project_file()` (`src/config.rs`) ne produit que `languages` et `package_manager`, et `aarshi add-lang` (`src/lang.rs`) réécrit `aarshi.toml` en entier à partir de cette même structure.

À trancher :

- L'extension d'`aarshi.toml` — écriture par `setup`, préservation par `add-lang`, schéma strict étendu (famille `setup`/`config`/`lang`/`templates`) — fait-elle partie de cette carte, ou d'une carte d'implémentation distincte ? Tant qu'elle n'est pas faite, la règle `config.project-file-incomplete` (`todo`) couvre la clé absente, conformément à la spec.
- La disparition de `Command::Check` au profit de l'alias visible de `status`, et la migration des tests `check` existants, relèvent-elles du périmètre de cette carte ? Le détail (plomberie, tests) est instruit par le ticket « Codes de sortie et absorption de `check` ».
- Y a-t-il d'autres travaux **amont** exigés par la spec (par ex. des fichiers de base que `setup` ne génère pas encore, ou un marqueur manquant) à intégrer au périmètre ?

## Resolution
