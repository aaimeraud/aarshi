---
labels: [wayfinder:grilling]
parent: 0014-map-implementation-status.md
blocked-by: [0015-grilling-perimetre-implementation.md]
status: closed
assignee: aaimeraud
---

# Modèle interne du moteur de `status`

## Question

Quelle structure de code pour `status` ? Notamment :

- **Le modèle des constats.** `Finding` (`rule`, `severity`, `summary`, `detail`, `location`), `Dimension` (six identifiants dans l'ordre canonique), `Severity` (`error`/`todo`), `Location` (`file`, `locale`, `key` ; `line` réservé, absent en v1). Où vivent les types et les 36 identifiants de règle : un module `src/status.rs`, un dossier `src/status/` éclaté par dimension, ou autre ?
- **L'organisation des contrôles.** Une fonction par dimension, par règle, ou une passe par objet ? Comment le moteur les itère, collecte tous les constats (la règle transverse interdit tout arrêt anticipé), puis trie (par `location.file` puis identifiant de règle) et compte.
- **Les primitives de lecture partagées** (texte, sans parseur TS) : extraction d'imports, clés de dictionnaire (aujourd'hui `dictionary::entries`, à étendre pour les doublons et les clés en trop), scan récursif des routes sous `app/[locale]/`, entrées de sitemap, marqueurs. Que reprend-on de `src/add.rs`, `src/dictionary.rs`, `src/check.rs`, et qu'introduit-on ?
- **Le seam d'extension.** Comment laisser ouverte l'extensibilité (contrôles déclarés par le dev, listée dans « Not yet specified ») sans l'implémenter ni complexifier le moteur v1 ?
- **Le point d'entrée.** Signature de `status::run(...)` : que reçoit-il (dossier, `Config`, options), que renvoie-t-il (modèle de rapport) et où sont calculés filtre, compteurs et code de sortie.

La **forme de sortie** (rendu humain, `--json`, wording) et la **plomberie des codes** sont tranchées par les tickets « Rendu humain, `--json` et wording des messages » et « Codes de sortie et absorption de `check` ».

## Resolution

**Un dossier `src/status/`, un module par dimension.** Le tracer tranche : le moteur vit dans un dossier `src/status/`, et chaque dimension apporte son fichier de contrôles.

- `src/status/mod.rs` : les types publics — `Dimension` (six identifiants canoniques, avec `Dimension::ALL` dans l'ordre d'affichage), `Severity` (`Error`/`Todo`, sérialisé `error`/`todo`), `Location` (`file`, `locale`, `key` en `Option`, l'absence est sérialisée « champ omis » ; `line` reste réservé), `Finding` (`rule: &'static str`, `severity`, `summary`, `detail: Option`, `location: Option` et un champ `dimension` non sérialisé), `Report` (les `Finding` triés + compteurs), les options CLI (`Options`: `dimensions`, `errors_only`, `json`) et le point d'entrée `run()`.
- `src/status/i18n.rs` : les quatre contrôles du tracer. Les tickets suivants ajoutent `config.rs`, `pages.rs`, `sections.rs`, `components.rs`, `dependencies.rs` sur le même modèle.
- `src/status/render.rs` : rendu humain et sérialisation `--json` (voir [0017](0017-grilling-rendu-json.md)).

**Les 36 identifiants de règle** vivent dans le module de leur dimension : chaque `Finding` porte son `rule: &'static str` littéral, aucun registre central n'est maintenu en code. La table de [0011](0011-grilling-contrat-sortie.md) reste la source de vérité ; les tests noirs écrivent les identifiants attendus, ce qui les rend stables.

**Organisation des contrôles** : une fonction collectrice par dimension, `collect(root: &Path, config: &Config, findings: &mut Vec<Finding>)`, enregistrée dans la constante `COLLECTORS: &[Collector]` de `mod.rs` (un `fn` par dimension). `status::run` itère la liste **sans aucun arrêt anticipé**, trie ensuite les constats par cle `(location.file ou "" en tête, identifiant de règle)` (tri stable : l'ordre d'insertion sert d'arbitre à égalité), puis compte par dimension. Le rendre d'un fichier illisible est absorbé (constat adapté ou absence de constat), il ne coupe jamais la collecte.

**Primitives de lecture partagées** : reprises telles quelles de `src/dictionary.rs` (`entries`, et `is_empty` élargi à « vide après `trim()` ») ; `listed_languages` (lecture textuelle de `export const languages` de `lib/i18n/config.ts`) quitte `src/check.rs` pour `status/i18n.rs`. Les extracteurs d'imports, entrées de sitemap, scans de routes et marqueurs seront introduits par leurs tickets de dimension, au moment de leur premier contrôle — pas d'abstraction anticipée.

**Seam d'extension** : la liste `COLLECTORS` est le seul point à modifier pour ajouter une dimension ; les contrôles déclarés par le développeur (« Not yet specified ») passeront plus tard par ce même seam, sans le spécialiser en v1.

**Point d'entrée** : `status::run(options: Options) -> Result<()>` reçoit le répertoire courant, lit `aarshi.toml` (`Config::for_project`) et renvoie `Err(status::Exit { code, message })` pour les codes `1` et `2` (voir [0018](0018-grilling-codes-sortie-alias-check.md)). Le rapport est imprimé par `run` (rendu humain ou JSON) avant le renvoi ; filtre, compteurs et verdict sont calculés dans `_Report_` (`counts` par dimension, `totals` global, `selected`/`shown` appliquant les filtres) pour que code de sortie et compteurs restent calculés sur le projet entier.
