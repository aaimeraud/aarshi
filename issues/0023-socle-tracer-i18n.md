---
labels: [wayfinder:implement]
parent: 0014-map-implementation-status.md
blocked-by: []
status: open
---

# Socle et tracer : surface `status`, moteur, sortie, dimension i18n partielle

## Tranche

Le tracer : `aarshi status` saisie de bout en bout, démontré par la dimension `i18n` (les quatre contrôles qu'`check` couvre déjà), avec **tout** l'outillage transverse :

- **Surface** ([0010](0010-grilling-surface-status.md)) : sous-commande `status` en dernier dans l'aide (`setup`, `add`, `add-lang`, `icons`, `status`), `check` = alias **visible** (mêmes flags, même sortie, même code), `Command::Check` supprimé ; `--dimension <nom>` répétable (virgules acceptées) validée par `clap` sur les six identifiants de [0013](0013-grilling-dimensions.md) ; `--errors-only` ; `about` = `Report what is missing or wrong in the Next.js project`.
- **Moteur** ([0016](0016-grilling-moteur-status.md)) : types `Finding` (`rule`, `severity`, `summary`, `detail`, `location`), `Severity` (`error`/`todo`), `Dimension` (six identifiants en ordre canonique), `Location` (`file`, `locale`, `key` ; `line` réservé) ; collecte **sans aucun arrêt anticipé** (règle transverse), tri d'affichage par `location.file` puis identifiant de règle, comptage par dimension. L'organisation détaillée (module unique vs dossier par dimension) est decidée ici.
- **Rendu humain** ([0011](0011-grilling-contrat-sortie.md)) : une ligne par dimension dans l'ordre canonique, `✓` quand la dimension est propre, constats indentés `error:`/`todo:` (ajout du helper `todo()` à `src/output.rs`, symétrique de `warning()`), verdict final en trois formes, `print_header()` conservé (et `AARSHI_NO_HEADER`), ligne `info: N findings hidden by --dimension` quand le filtre masque au moins un constat.
- **Codes de sortie** ([0018](0018-grilling-codes-sortie-alias-check.md)) : `0` (aucune `error`), `1` (≥ 1 `error`), `2` (`aarshi.toml` absent/illisible/invalide, message sur stderr, rien sur stdout) ; hors projet : `error: no aarshi.toml in this folder; nothing to check` remplace le message actuel de `check`. Le plumbing sera tranché ici (type d'erreur dédié ou code renvoyé par `status::run`), en laissant les autres sous-commandes inchangées.
- **`--json`** ([0011](0011-grilling-contrat-sortie.md)) : `serde_json` ajouté à `Cargo.toml` ; `format_version: 1`, `summary` global, six dimensions toujours présentes, `findings` `rule`/`severity`/`summary`/`detail`/`location` avec options omises ; document unique sans bannière ni couleur, **rien sur stdout** en cas d'arrêt.
- **Règles i18n du tracer** ([0006](0006-grilling-invariants-i18n.md)) : `i18n.languages-mismatch`, `i18n.dictionary-missing`, `i18n.key-missing`, `i18n.key-empty` (vide élargi à « vide après `trim()` ») ; la référence reste le dictionnaire de `languages[0]`.
- **Tests** ([0019](0019-grilling-verification-status.md)) : fixture de « projet généré » minimale (fabrique dédiée, sans passer par un vrai `setup` si elle suffit), migration du test `check_reports_empty_translations_until_they_are_filled` (la traduction vide devient un `todo`, donc verdict et code de sortie changent), conventions de wording des `summary` posées ici.

Cette tranche **clôt les grillings [0016](0016-grilling-moteur-status.md), [0017](0017-grilling-rendu-json.md) et [0018](0018-grilling-codes-sortie-alias-check.md)** en rédigeant leurs résolutions.

## Critères d'acceptance

- [ ] Sur une fixture, le rapport humain reproduit la forme de l'exemple de rendu de 0011 (dimensions toujours listées, `✓`, constats indentés, verdict, ligne d'info sous filtre), ET/FR non concerné.
- [ ] `aarshi status` et `aarshi check` produisent la même sortie, le même code ; `status` est en dernier dans l'aide et `check` apparaît comme alias ; une valeur `--dimension` inconnue est une erreur d'usage (code 2).
- [ ] Codes de sortie vérifiés par tests : propre → `0`, `todo` seuls → `0`, ≥ 1 `error` → `1`, hors projet → `2` avec rien sur stdout.
- [ ] `--json` comparé champ par champ sur une fixture (schéma de 0011 exact, options omises) ; le schema reste strict sur stdout en cas d'arrêt (code 2).
- [ ] La résolution de 0016 documente le modèle retenu (emplacements des types et des 36 identifiants), celle de 0017 les conventions de wording, celle de 0018 le plumbing des codes.
