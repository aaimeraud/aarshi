---
labels: [wayfinder:grilling]
parent: 0014-map-implementation-status.md
blocked-by: [0015-grilling-perimetre-implementation.md]
status: closed
assignee: aaimeraud
---

# Codes de sortie et absorption de `check`

## Question

- **Plomberie des codes.** `main()` (`src/main.rs`) mappe aujourd'hui toute erreur sur le code `1` et imprime `error:`. Comment distinguer `0` (aucune `error`), `1` (au moins une `error`) et `2` (`status` ne peut pas analyser : `aarshi.toml` absent, illisible ou invalide, message sur stderr, rien sur stdout) ? Type d'erreur dédié, code de sortie renvoyé par `status::run`, ou autre. Quel effet sur les autres sous-commandes, qui gardent leur comportement actuel ?
- **Alias `check`.** `#[command(visible_alias = "check")]` sur `status`, suppression de `Command::Check`, `about` (`Report what is missing or wrong in the Next.js project`) et place de `status` en dernier dans l'aide (`setup`, `add`, `add-lang`, `icons`, `status`).
- **Message hors projet.** `error: no aarshi.toml in this folder; nothing to check` sur stderr, code `2` — il remplace le message actuel de `check` (`not an aarshi project: …`) et celui des autres commandes reste-t-il inchangé ?
- **Migration des tests.** Le test `check_reports_empty_translations_until_they_are_filled` change de comportement : la traduction vide devient un `todo`, donc `status` sort `0` avec un rapport différent. Quels tests réécrire, lesquels ajouter, et faut-il un test dédié par code de sortie ?

## Resolution

**Plomberie : type d'erreur dédié.** Un `pub struct Exit { code: i32, message: String }` dans `src/status/mod.rs`, qui implémente `Display` et `std::error::Error` et est renvoyé dans la `Result` de `status::run`. `main()` interroge la chaîne d'erreurs avec `downcast_ref::<status::Exit>()` **avant** son traitement générique : si la conversion réussit, il imprime `error: <message>` sur stderr **quand `message` n'est pas vide** (il reste vide quand le rapport a déjà tout dit, cas du code `1`) puis sort avec `exit.code` ; sinon le comportement anti-régression est intact — toute autre sous-commande échoue toujours en `error: …` avec le code `1`. Concrètement :

- `status::run` démarre par la garde projet : pas d'`aarshi.toml` → `Err(Exit { code: 2, message: "no aarshi.toml in this folder; nothing to check" })`, **rien sur stdout** ;
- après l'impression du rapport (humain ou JSON), le résultat se décide sur les totaux du projet complet : `errors > 0` → `Err(Exit { code: 1, message: "" })`, sinon `Ok(())`.

Conséquence déplacée : l'en-tête ne peut plus être imprimé globalement avant la sous-commande (il polluerait stdout en cas d'arrêt code `2` et en `--json`). `run()` de `main.rs` n'appelle `print_header()` que pour les commandes autres que `status` ; `status::run` l'affiche lui-même, exclusivement en mode humain et après la garde projet.

**Message hors projet.** `status` et `check` remplacent `not an aarshi project: no aarshi.toml in this folder` par `error: no aarshi.toml in this folder; nothing to check` (code `2`). Les autres sous-commandes gardent leur message actuel inchangé.

**Alias `check`.** La variante `Status` porte `#[command(visible_alias = "check")]` et reste déclarée **en dernier** dans l'enum (`setup`, `add`, `add-lang`, `icons`, `status`) ; `Command::Check` et `src/check.rs` disparaissent. L'`about` est `Report what is missing or wrong in the Next.js project`. `clap` donne à l'alias exactement les mêmes flags : `check_gives_the_same_report_as_status` vérifie stdout, stderr et le code identiques sur une fixture, et `the_help_lists_status_last_with_check_as_alias` vérifie l'ordre d'aide, l'absence de l'ancien texte de `check` et la présence de l'alias.

**Migration des tests.** `check_reports_empty_translations_until_they_are_filled` devient `status_reports_empty_translations_until_they_are_filled` (sur un projet généré par le vrai `setup` + `add-lang de`) : après ajout de `de`, trois traductions vides → code `0` et verdict `✓ project is conformant (3 to do)` ; dictionnaires remplis → code `0` et `✓ project is conformant`. Les autres codes ont chacun leur test : propre → `0` (`status_lists_every_dimension_and_verifies_them_all_clean`), `todo` seuls → `0` (`status_todos_alone_still_exit_zero`), au moins une `error` → `1` (`status_reproduces_the_contract_render_on_a_fixture`), hors projet → `2`, rien sur stdout (`status_exits_two_outside_a_project_and_prints_nothing`, avec la variante `--json` et la même phrase sous l'alias `check`), valeur `--dimension` inconnue → erreur d'usage `clap`, code `2` (`an_unknown_dimension_is_a_usage_error`).
