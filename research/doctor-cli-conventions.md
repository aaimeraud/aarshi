# Recherche : conventions des CLI pour une commande de diagnostic

> **Note sur le chemin.** Le dépôt n'avait aucune convention de fichier de recherche (les tickets `issues/*.md` portent le label `wayfinder:research` mais ne renvoient à aucun dossier `research/`). Le chemin retenu ici est `research/doctor-cli-conventions.md` : un dossier `research/` à la racine, un fichier par ticket, nommé d'après le sujet du ticket `issues/0009-research-doctor-cli.md`. C'est un choix, pas une contrainte préexistante.

**Question traitée** (ticket `issues/0009-research-doctor-cli.md`) : comment des CLI comparables conçoivent-ils une commande de diagnostic (`doctor`, `status`, `check`, `verify`) ? Précisément : (1) les catégories de vérifications, (2) la distinction de gravité et la façon dont le code de sortie en découle, (3) la sortie machine proposée et sa forme.

## Réponse directe

### 1. Catégories de vérifications

- **`npm doctor` est organisé en groupes nommés et fixes** : `connection`, `registry`, `versions`, `environment`, `permissions`, `cache`. Les noms de groupes sont littéralement listés dans le commando, et chaque vérification du groupe imprime `Ok` / `Not ok`. Une vérification peut être ciblée par son nom passé en argument. Source : `https://raw.githubusercontent.com/npm/cli/latest/lib/commands/doctor.js` (tableau de `groups` et sortie `log.info`/`log.warn`) ; documentation : `https://docs.npmjs.com/cli/v10/commands/npm-doctor`.
- **`brew doctor` répartit le diagnostic en « checks » nommés et énumérables** : `--list-checks` liste les vérifications, et des `diagnostic_check` peuvent être passés en arguments nommés pour n'en exécuter qu'une partie. Source : `https://docs.brew.sh/Manpage` (section `doctor`).
- **Les catégories récurrentes** couvrent : fichiers/binaires attendus et versions (`npm doctor` `versions`, `rustup check`), état du système et de l'environnement (`npm doctor` `environment`), configuration (`prisma validate`, `terraform validate`), état de dépendances (`flutter doctor`, `npm doctor` `cache`/`permissions`), et état d'un service distant (`gh auth status`, `npm doctor` `connection`/`registry`). Sources : `https://docs.npmjs.com/cli/v10/commands/npm-doctor` ; `https://docs.brew.sh/Manpage` ; `https://www.prisma.io/docs/orm/reference/prisma-cli-reference` ; `https://developer.hashicorp.com/terraform/cli/commands/validate` ; `https://cli.github.com/manual/gh_auth_status`.
- **Un diagnostic croisé config ↔ état réel existe mais reste l'exception** : `terraform validate` valide la configuration d'un module contre les règles du langage (pas contre un état de déploiement) et retourne des diagnostics localisés par plage de lignes. Source : `https://developer.hashicorp.com/terraform/cli/commands/validate`.
- **Le pattern « commandes séparées plutôt qu'un `doctor` » existe** : Prisma n'a pas de `doctor` unique ; il expose `validate`, `format --check`, `migrate status`, `debug` séparés. Idem, `cargo` n'a **aucune** sous-commande `doctor` (les sous-commandes sont General/Build/Manifest/Package/Publishing/Report/Deprecated). Sources : `https://www.prisma.io/docs/orm/reference/prisma-cli-reference` ; `https://doc.rust-lang.org/cargo/commands/index.html`.

### 2. Gravité et code de sortie

- **Deux à quatre niveaux de sévérité** selon les outils ; le plus courant est `error` / `warning` (+ `info`/`style`/`note`).
  - ShellCheck : `error`, `warning`, `info`, `style`, filtrables par `-S`/`--severity`. Source : `https://raw.githubusercontent.com/koalaman/shellcheck/master/shellcheck.1.md`.
  - ESLint : niveaux `0/1/2` = `off`/`warn`/`error`. Source : `https://eslint.org/docs/latest/use/command-line-interface`.
  - Terraform : `severity` d'un diagnostic ∈ {`error`, `warning`}. Source : `https://developer.hashicorp.com/terraform/cli/commands/validate`.
  - SARIF 2.1.0 : `result.level` et `notification.level` ∈ `[none, note, warning, error]`, défaut `warning`. Source : `https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json`.
- **Le code de sortie est binaire « quelque chose à signaler ou non », pas proportionnel au nombre de problèmes.**
  - `brew doctor` : « Will exit with a non-zero status if any potential problems are found. » Source : `https://docs.brew.sh/Manpage`.
  - ESLint : `0` si aucun problème sous le seuil (`--max-warnings`), `1` si ≥ 1 erreur ou dépassement du seuil d'avertissements. Source : `https://eslint.org/docs/latest/use/command-line-interface`.
  - ShellCheck : `0` aucun problème, `1` problèmes trouvés. Source : `https://raw.githubusercontent.com/koalaman/shellcheck/master/shellcheck.1.md`.
- **Un code de sortie spécial distingue l'erreur interne/config de l'échec de diagnostic.**
  - ESLint : `2` pour erreur de configuration ou erreur interne. Source : `https://eslint.org/docs/latest/use/command-line-interface`.
  - ShellCheck : `2` fichiers non traitables, `3` syntaxe d'invocation invalide, `4` options invalides. Source : `https://raw.githubusercontent.com/koalaman/shellcheck/master/shellcheck.1.md`.
  - `gh` : `0` succès, `1` échec quelconque, `2` annulé, `4` authentification requise. Source : `https://cli.github.com/manual/gh_help_exit-codes`.
- **Certains outils distinguent un « avertissement » non bloquant dans leur type interne, sans que cela remonte forcément au shell.** Flutter définit `enum ExitStatus { success, warning, fail, killed }` et `DoctorCommand` retourne `ExitStatus.warning` quand un diagnostic échoue ; mais le runner mappe les erreurs d'usage sur `64`, un `ToolExit` sur `1`, un crash sur `1` — la nuance `warning` sert d'abord à l'analytics, pas à un code de sortie dédié. Sources : `https://raw.githubusercontent.com/flutter/flutter/master/packages/flutter_tools/lib/src/commands/doctor.dart` ; `https://raw.githubusercontent.com/flutter/flutter/master/packages/flutter_tools/lib/src/runner/flutter_command.dart` ; `https://raw.githubusercontent.com/flutter/flutter/master/packages/flutter_tools/lib/src/runner.dart`.
- **`rustup check` a un code de sortie dédié « mises à jour trouvées » : `100`**, contre `0` si tout est à jour. Source : `https://raw.githubusercontent.com/rust-lang/rustup/master/CHANGELOG.md` (1.29.0, PR #4694 ; section « Exit status » ajoutée par PR #4696).

### 3. Sortie machine

- **`--json` « maison » avec un schéma versionné et des diagnostics structurés** — c'est le modèle le plus directement transposable :
  - `terraform validate -json` : `valid` (bool), `error_count`, `warning_count`, et `diagnostics[]` avec `severity` ∈ {`error`,`warning`}, `summary`, `detail`, `range.{filename,start,end}`, plus `format_version` `"1.0"` (mineur = rétrocompatible). Sources : `https://developer.hashicorp.com/terraform/cli/commands/validate`.
  - ESLint formatter `json` : par fichier `filePath`, `messages[].{ruleId,severity,message,line,column,messageId,endLine,endColumn,fix}`, plus `errorCount`, `warningCount`, `usedDeprecatedRules`. Sources : `https://eslint.org/docs/latest/use/formatters/`.
- **SARIF 2.1.0** est le format standard inter-outils pour les diagnostics : `version` `"2.1.0"` (requis), `runs[]`, chaque résultat avec `level` ∈ `[none, note, warning, error]`, `ruleId`/`ruleIndex` pour rattacher au catalogue de règles, et `invocation.exitCode` + `invocation.executionSuccessful` (booléen requis). Source : `https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json`. ESLint **ne** fournit pas de formatter SARIF intégré (formatters intégrés : `stylish`, `json`, `json-with-metadata`, `html`). Source : `https://eslint.org/docs/latest/use/formatters/`.
- **JSON-lines / flux d'objets** est le modèle de `cargo` : `--message-format=json` émet un objet JSON par ligne (`reason`, …, terminé par un objet `build-finished`) ; `cargo metadata --format-version` est un document unique versionné et stable. Sources : `https://doc.rust-lang.org/cargo/reference/external-tools.html` ; `https://doc.rust-lang.org/cargo/commands/index.html`.
- **Tous les diagnostics n'ont pas de sortie machine.** `npm doctor` ne documente pas de `--json` (le code n'imprime que des lignes humaines `Ok`/`Not ok`). Sources : `https://docs.npmjs.com/cli/v10/commands/npm-doctor` ; `https://raw.githubusercontent.com/npm/cli/latest/lib/commands/doctor.js`. De même `flutter doctor` réserve `--machine` à `--version` (sinon code de sortie `2`), donc pas de diagnostic machine. Sources : `https://raw.githubusercontent.com/flutter/flutter/master/packages/flutter_tools/lib/src/runner/flutter_command_runner.dart`.
- **`gh auth status` montre un piège de contrat** : la commande sort en erreur `1` en mode humain (message sur stderr), mais sort toujours `0` avec `--json` sauf erreur fatale — le JSON est la charge utile, l'état est dans le document. Source : `https://cli.github.com/manual/gh_auth_status`.

## Synthèse pour `aarshi status`

1. **Surface** : une seule commande de diagnostic en lecture seule, avec des **catégories nommées et stables** (à l'image des `groups` de `npm doctor` et des `--list-checks` de `brew doctor`) — ici : présence des fichiers attendus, câblage croisé, config `aarshi.toml`, dictionnaires i18n, routes `app/[locale]/`.
2. **Surface** : `status` absorbe `check` comme alias ; le nom reste `status`, la sémantique ne change pas. Aucune écriture dans le projet analysé (invariant de la carte).
3. **Surface** : deux gravités seulement suffisent et correspondent au contrat de la carte — `error` (conformité cassée) et `todo` (reste-à-faire attendu) ; ne pas inventer un troisième niveau que personne ne consommerait.
4. **Surface** : permettre de cibler une catégorie par argument nommé (comme `brew doctor <check>`) est une extension naturelle, mais hors v1 si le temps manque.
5. **Gravité → sortie** : le code de sortie doit être **non nul si et seulement si ≥ 1 `error`** ; un rapport qui ne contient que des `todo` sort `0` (un `todo` n'est pas un échec, c'est l'état attendu du dev).
6. **Gravité → sortie** : réserver un **code dédié à l'erreur interne/config** (à l'image d'ESLint `2` et ShellCheck `2`), distinct de « diagnostic a signalé des erreurs » (`1`) et de « succès » (`0`) — par ex. `aarshi.toml` illisible ou invalide.
7. **Gravité → sortie** : rester sur un code binaire pour les diagnostics (comme `brew doctor`, ESLint, ShellCheck) ; ne pas proportionner le code au nombre d'erreurs. Le détail vit dans la sortie, pas dans le code.
8. **Sortie humaine** : réutiliser les helpers existants (`error:`, `warning:`, `✓`, `info:`) et `print_header` (respectant `AARSHI_NO_HEADER`) ; rapport **groupé par dimension**, une ligne par constat, comme `npm doctor`/`brew doctor`.
9. **Sortie machine** : offrir `--json` avec un **schéma maison versionné** (`format_version` à la Terraform) et une liste de diagnostics structurés `{ id/ruleId, severity, summary, detail, location }`.
10. **Sortie machine** : aligner les sévérités JSON sur un enum explicite `error`/`todo` (documenté), et exposer des compteurs `error_count`/`todo_count` en plus de la liste — c'est exactement la forme de `terraform validate -json`.
11. **Sortie machine** : garantir **du JSON strict sur stdout** (pas de bruit d'en-tête/logs sur stdout en mode `--json`), le mode humain seul imprimant l'en-tête décoré.
12. **Sortie machine** : avec `--json`, privilégier un **code de sortie cohérent avec le rapport** (ou documenter explicitement le choix inverse de `gh auth status`) pour éviter le piège « JSON valide mais code 0 alors que des `error` existent ».
13. **Sortie machine** : **SARIF n'est pas requis en v1** — aucun outil comparable cité ne l'a par défaut (ESLint ne l'intègre pas, Terraform a son propre `format_version`) ; à considérer comme sortie future si un besoin IDE/CI apparaît.
14. **Sortie machine** : donner à chaque constat un **identifiant stable** (`ruleId`-like, ex. `missing-dictionary`, `unwired-section`) pour permettre les filtres, la déduplication et la stabilité des tests — repris du modèle ESLint (`ruleId`) et SARIF (`ruleId`/`ruleIndex`).
15. **Contrat** : valider la sortie JSON en test (schéma + codes de sortie 0/1/2) plutôt que de graver le texte humain, qui reste libre d'évoluer.

## Sources

- **npm doctor (doc + code)** : `https://docs.npmjs.com/cli/v10/commands/npm-doctor` ; `https://raw.githubusercontent.com/npm/cli/latest/lib/commands/doctor.js`
- **ESLint (CLI, formatters)** : `https://eslint.org/docs/latest/use/command-line-interface` ; `https://eslint.org/docs/latest/use/formatters/`
- **Terraform `validate -json`** : `https://developer.hashicorp.com/terraform/cli/commands/validate`
- **SARIF 2.1.0 (schéma OASIS)** : `https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json` (id `docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json`)
- **Homebrew `doctor` (manpage)** : `https://docs.brew.sh/Manpage`
- **Flutter tool (doctor, runner, exit codes)** : `https://raw.githubusercontent.com/flutter/flutter/master/packages/flutter_tools/lib/src/commands/doctor.dart` ; `https://raw.githubusercontent.com/flutter/flutter/master/packages/flutter_tools/lib/src/runner/flutter_command.dart` ; `https://raw.githubusercontent.com/flutter/flutter/master/packages/flutter_tools/lib/src/runner.dart` ; `https://raw.githubusercontent.com/flutter/flutter/master/packages/flutter_tools/lib/src/runner/flutter_command_runner.dart`
- **rustup `check` (CHANGELOG)** : `https://raw.githubusercontent.com/rust-lang/rustup/master/CHANGELOG.md`
- **GitHub CLI (codes de sortie, `auth status`)** : `https://cli.github.com/manual/gh_help_exit-codes` ; `https://cli.github.com/manual/gh_auth_status`
- **Prisma CLI (référence)** : `https://www.prisma.io/docs/orm/reference/prisma-cli-reference`
- **Cargo (sous-commandes, sortie machine)** : `https://doc.rust-lang.org/cargo/commands/index.html` ; `https://doc.rust-lang.org/cargo/reference/external-tools.html`
- **ShellCheck (manpage)** : `https://raw.githubusercontent.com/koalaman/shellcheck/master/shellcheck.1.md`

## Points incertains

- **Flutter** : je n'ai pas localisé le mapping exact `ExitStatus` → code de sortie shell. Le code montre que `DoctorCommand` retourne `ExitStatus.warning` et que le runner mappe usage→`64`, `ToolExit`→`1`, crash→`1` ; la valeur shell finale d'un `ExitStatus.warning` de `doctor` n'a pas été observée sur la machine. Les faits cités se limitent donc à ce que le code montre explicitement.
- **npm doctor** : l'absence de `--json` est déduite de la doc et du code du commando ; je n'ai pas vérifié toutes les versions majeures de npm.
- **SARIF** : j'ai lu le schéma officiel pour les champs cités ; je n'ai pas vérifié la conformité d'un générateur concret.
- **brew doctor** : la liste exacte des checks varie selon les versions de Homebrew ; la manpage citée décrit `--list-checks` sans énumérer la liste.
- Les URLs GitHub pointent vers `master`/`latest` ; les versions stables publiées peuvent différer de ce que j'ai lu.
- **`cargo`** : l'absence de sous-commande `doctor` est vérifiée sur l'index des commandes, mais une extension tierce pourrait en fournir une.
