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
- Notes de recherche : dossier `research/` à la racine du dépôt (un fichier par ticket de recherche) ; chaque ticket `wayfinder:research` pointe vers son fichier.
- Exécution : cet effort autorise d'enchaîner plusieurs tickets dans une même session (demande explicite) ; l'agent ouvre le ticket suivant dès qu'une résolution est enregistrée. Chaque résolution est commitée séparément pour pouvoir s'arrêter à tout moment.
- Règle transverse : toute attente non satisfaite est un **constat** ; `status` ne s'arrête jamais sur un fichier-clé absent (contrairement à `check` aujourd'hui) — il continue et rapporte tout ce qu'il peut.

## Decisions so far

<!-- l'index : une ligne par ticket fermé, avec son gist et le lien vers le détail -->

- [Recherche : comment les CLI conçoivent une commande de diagnostic (`doctor`/`status`)](0009-research-doctor-cli.md) : catégories nommées et stables ; gravité `error`/`warning` ; code de sortie binaire (non nul si et seulement s'il y a ≥ 1 `error`) + code dédié à l'erreur interne ; `--json` versionné à identifiants de règle stables ; SARIF hors v1.
- [Surface de `status` et absorption de `check`](0010-grilling-surface-status.md) : terme canonique « dimension » ; `check` = alias **visible** de `status` (mêmes flags/sortie/code, sans message de dépréciation) ; filtre `--dimension <nom>` répétable et validé par `clap` ; `--errors-only` seul mode court ; hors projet, arrêt avec `no aarshi.toml in this folder; nothing to check` ; `status` en dernier dans l'aide ; doc déléguée au ticket de livraison.
- [Invariants i18n : lesquels `status` vérifie-t-il, et à quelle gravité ?](0006-grilling-invariants-i18n.md) : config i18n vérifiée en entier (`languages` ↔ `aarshi.toml`, `defaultLocale`, `direction`, `index.ts`) = `error` ; dictionnaires : fichier absent / clé manquante / doublon = `error`, valeur vide (blancs inclus) / clé en trop / dictionnaire orphelin = `todo` ; l'attente des clés attendues d'une vue est rattachée au ticket pages/sections.
- [Invariants pages, routes, sections et composants : lesquels `status` vérifie-t-il ?](0007-grilling-invariants-pages.md) : vue ↔ route (deux sens) + clé `"<page>.title"` + sitemap + import de vue = `error` ; sections : import pendant / clé / marqueur = `error`, section orpheline = `todo` ; icône `.svg` sans `.tsx` = `todo` (`.tsx` sans `.svg` ignoré) ; les imports connus doivent résoudre (`error`), les composants `custom` non utilisés ne sont pas signalés ; les clés attendues viennent du filesystem.
- [Invariants config/projet et outillage : lesquels `status` vérifie-t-il ?](0008-grilling-invariants-config.md) : `aarshi.toml` étendu (`ui`, `icons`, `cache_components`, `site_url` ; **pas** `agent_feedback`) et options contrôlées contre le projet ; fichiers/dossiers de base, redirect `next.config.ts` et marqueur sitemap = `error` ; dépendances non déclarées et lockfile incohérent = `error`, `node_modules` absent et clé de config manquante = `todo` ; la ligne de télémétrie `.env` n'est pas vérifiée.
- [Dimensions canoniques de `status` : liste et identifiants stables](0013-grilling-dimensions.md) : six dimensions dans cet ordre — `config`, `i18n`, `pages`, `sections`, `components`, `dependencies` ; un constat appartient à la dimension de l'**objet** concerné (pas de dimension `links`) ; les identifiants servent de valeurs à `--dimension` et de clés à `--json`.
- [Contrat de sortie : rendu humain, `--json`, codes de sortie](0011-grilling-contrat-sortie.md) : rapport par dimension (`✓` si propre, constats indentés `error:`/`todo:`, verdict final) ; ordre des constats par fichier puis règle ; codes `0`/`1`/`2` (le `2` = projet non analysable) ; `--json` versionné (`format_version`, `summary` global, toujours les six dimensions, `findings` `rule`/`severity`/`summary`/`detail`/`location`) ; `--dimension` et `--errors-only` n'agissent que sur l'affichage ; 36 identifiants de règle stables.

## Not yet specified

- Extensibilité : `status` doit-il permettre au dev de déclarer ses propres contrôles (registre, config) ?
- Prise en compte des commandes aarshi futures dans le catalogue d'invariants (par ex. un futur `aarshi update`).
- Versionnage explicite du scaffolding généré et migration (au-delà du cas « clé absente d'`aarshi.toml` », tranché dans le catalogue config).

## Out of scope

- Analyse sémantique TypeScript/AST (parser les imports, vérifier qu'une section est réellement rendue) : reportée ; la spec v1 s'en tient à la présence et au câblage. Pourra faire l'objet de sa propre carte.
- `--fix` (réparation automatique) : écarté de cette spec.
- Projets Next.js sans `aarshi.toml` (mode dégradé) : le périmètre est le projet généré par aarshi.
