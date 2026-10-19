---
labels: [wayfinder:map]
status: closed
parent: 0005-map-status.md
---

# Carte : implémentation de `aarshi status`

## Destination

`aarshi status` **implémenté et livré** : la commande de diagnostic en lecture seule décrite par `issues/0005-map-status.md` (six dimensions, 36 identifiants de règle, rapport humain groupé et verdict final, `--json` versionné `format_version: 1`, codes de sortie `0`/`1`/`2`, `check` comme alias visible), couverte par des tests noirs et documentée en EN + FR.

## Notes

- **La spec est figée.** Elle vit dans `issues/0005-map-status.md` et les tickets `0006`–`0013`, tous fermés. Cette carte ne rouvre **aucune** décision de spec : elle tranche des questions d'**implémentation**.
- Hérite des décisions de `issues/0005-map-status.md`, `issues/0001-map.md` et `issues/0004-map-implementation.md`. Les décisions héritées **renvoient** aux tickets fermés (liens) au lieu d'être recopiées : une seule source par décision.
- Domaine : CLI Rust (`clap`, `colored`), lecture seule du projet généré. Base la plus proche : `src/check.rs` (absorbé par `status`), `src/output.rs` (helpers + en-tête, à compléter d'un `todo()`), `src/config.rs`, `src/dictionary.rs`, `src/project.rs`, `src/add.rs` (routes, vues, imports), `src/main.rs` (déclaration `clap`), `tests/cli.rs` (28 tests noirs, `AARSHI_NO_HEADER=1`).
- **Contraintes déjà fixées par la spec** (elles cadrent les tickets, non rediscutables) : six dimensions et leur ordre canonique ; 36 identifiants de règle ; gravités `error`/`todo` ; tri des constats par `location.file` puis identifiant de règle ; `info: N findings hidden by --dimension` ; codes `0`/`1`/`2` (`2` = projet non analysable, rien sur stdout) ; `--json` avec `format_version` 1, six dimensions toujours présentes et `summary` global ; `--dimension` et `--errors-only` n'agissent que sur l'affichage ; lecture seule stricte ; pas d'analyse TS/AST ; `check` = alias visible de `status`, sans message de dépréciation.
- Compétences : `grilling` (+ `domain-modeling`) par défaut ; `research` si un ticket l'exige (aucun prévu).
- Conventions du dépôt : issues en français, sortie CLI en anglais, docs en paires EN + FR ; version control GitButler (`but`).
- Exécution : un ticket résolu par session (la recherche exceptée) ; chaque résolution commitée séparément ; l'index « Decisions so far » est mis à jour après chaque ticket.
- **Règle transverse** (héritée de la spec) : une attente non satisfaite est un **constat** ; un fichier-clé absent n'arrête jamais `status`.

## Decisions so far

<!-- une ligne par ticket fermé, avec son gist et le lien vers le détail -->

- **Hérite de la spec** : [carte `status`](0005-map-status.md) et ses tickets fermés — [invariants i18n](0006-grilling-invariants-i18n.md), [invariants pages/routes/sections/composants](0007-grilling-invariants-pages.md), [invariants config/outillage](0008-grilling-invariants-config.md), [recherche doctor/status](0009-research-doctor-cli.md), [surface et absorption de `check`](0010-grilling-surface-status.md), [contrat de sortie](0011-grilling-contrat-sortie.md), [livraison de la spec](0012-grilling-livraison-spec.md), [dimensions canoniques](0013-grilling-dimensions.md).
- [Amont : écriture étendue d'`aarshi.toml`](0022-amont-options-config.md) : `setup` écrit les six clés, `add-lang` les préserve, le schéma étendu reste tolérant à leur absence — [résolution de 0015](0015-grilling-perimetre-implementation.md), fermé.
- [Socle et tracer](0023-socle-tracer-i18n.md) : surface `status` (alias visible `check`, `--dimension`, `--errors-only`), moteur sous `src/status/` (modèle du moteur dans [0016](0016-grilling-moteur-status.md), rendu et wording dans [0017](0017-grilling-rendu-json.md), plomberie des codes dans [0018](0018-grilling-codes-sortie-alias-check.md) — les trois fermés), rendu humain, codes `0`/`1`/`2`, `--json` v1, quatre règles i18n du tracer.
- [Dimension `config`](0024-dimension-config.md) : fichiers et dossiers de base, marqueurs, redirect, options persistées contrôlées, import cassé d'un fichier de base.
- [Dimension `i18n` complète](0025-dimension-i18n.md) : `defaultLocale`, `direction`, `index.ts`, doublon, clé en trop, dictionnaire orphelin.
- [Dimension `pages` + primitives partagées](0026-dimension-pages.md) : vues ↔ routes (deux sens), sitemap, clé `"<page>.title"`, import de vue ; primitives (scan récursif des routes, extraction d'imports, sitemap, inventaire vues/sections) et conventions anti-double-rapport posées en addendum de [0016](0016-grilling-moteur-status.md).
- [Dimension `sections`](0027-dimension-sections.md) : import pendant, clés, marqueurs de vue, orphelines, imports cassés.
- [Dimension `components`](0028-dimension-components.md) : icônes `.svg` ↔ `.tsx`, imports cassés dans `components/` ; résolution des imports connus rangée dans la dimension de l'objet fautif (déjà couverte par 0024–0027).
- [Dimension `dependencies`](0029-dimension-dependencies.md) : paquets non déclarés (imports « bare » des fichiers connus), lockfile ↔ `package_manager`, `node_modules` absent.
- [Recette transverse](0030-recette-transverse.md) : matrice des 36 règles figée dans [0019](0019-grilling-verification-status.md) (fermé), règle « un constat par objet » (fichiers i18n auto-rapportés exclus de `config.file-missing`), invariants de sortie figés par tests.
- [Doc de `status`](0031-doc-status.md) : README et CONVENTIONS mise à jour EN + FR en parité, catalogue des invariants par dimension — [résolution de 0021](0021-grilling-doc-status.md), fermé.

## Not yet specified

- Extensibilité : `status` doit-il permettre au dev de déclarer ses propres contrôles (registre, config) ?
- Versionnage explicite du scaffolding généré et migration (au-delà du cas « clé absente d'`aarshi.toml` »).
- Prise en compte des commandes aarshi futures dans le catalogue d'invariants (par ex. un futur `aarshi update`).

## Out of scope

- Analyse sémantique TypeScript/AST : reportée par la spec ; `status` s'en tient à la présence et au câblage.
- `--fix` (réparation automatique).
- Projets Next.js sans `aarshi.toml` (le périmètre est le projet généré par aarshi).
- Sortie SARIF.
