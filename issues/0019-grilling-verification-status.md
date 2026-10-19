---
labels: [wayfinder:grilling]
parent: 0014-map-implementation-status.md
blocked-by: [0016-grilling-moteur-status.md, 0017-grilling-rendu-json.md]
status: closed
---

# Vérification : fixtures, couverture des 36 règles, tests `--json`

## Question

Comment le comportement de `status` est-il tenu par les tests ?

- **Fixtures.** Comment fabriquer les « projets cassés » : partir d'un projet généré par `setup` puis le muter (route manquante, clé supprimée, import cassé, lockfile incohérent…), ou écrire une fabrique dédiée ? Où placer les helpers : `tests/cli.rs` ou fichiers séparés (par dimension) ? Faut-il une variante de `Sandbox` qui produit un projet minimal sans passer par `setup` ?
- **Couverture.** Les 36 règles sont-elles couvertes une par une par un test noir, ou seulement les cas structurants (les six dimensions, le cas propre, plusieurs `error` et `todo` mêlés) ? À quel niveau de détail : message exact, identifiant de règle, ordre, code de sortie.
- **Contrat `--json`.** Comment figer le schéma : comparaison structurée du document, golden file, ou assertions ciblées par champ ? Où vérifier `format_version`, `summary` global sous `--dimension`/`--errors-only`, et l'absence de sortie sur stdout en cas d'arrêt.
- **Unités.** Faut-il des tests unitaires sur les primitives de lecture du moteur (extraction d'imports, clés, sitemap) en plus des tests noirs ?

## Resolution

**Fixtures.** Fabriques dédiées, pas de mutation d'un projet `setup` : chaque bloc de `tests/cli.rs` définit ses fabriques faites main, fidèles à ce que le générateur écrit pour les fichiers qu'elles posent (`i18n_project`, `pages_project`, `sections_project`, `dependencies_project`, et la vraie `FIXTURE_CONFIG`/`FIXTURE_INDEX` de `setup`), plus des mutateurs ponctuels (`write_dictionary`, `write_view`, `write_route`, `write_sitemap`, `write_section`…). `setup` reste appelé une fois en bout-en-bout pour le raccord réel avec le générateur (`status_reports_empty_translations_until_they_are_filled`). La variante « projet minimal sans `setup` » demandée par la question existe : [0030](0030-recette-transverse.md) réduit le projet au seul `aarshi.toml` (`recipe_a_project_carried_to_nothing_still_produces_the_full_report`).

**Niveau de détail.** Règle + gravité + **message mot pour mot**, partout où le constat porte un vocabulaire figé (0017) ; les rendus exacts (`status_reproduces_the_contract_render_on_a_fixture`, `status_lists_every_dimension_and_verifies_them_all_clean`, `status_todos_alone_still_exit_zero` et le bloc 0030) figent l'ordre canonique des dimensions, l'indentation, les `✓`, le verdict et le pluriel de `amount` ; les codes de sortie `0`, `1`, `2` ont chacun leur test (0024/0028, alias `check` en plus). Le tri `location.file` puis identifiant de règle, arbitré par l'insertion à égalité, est figé par `status_sorts_a_mixed_i18n_dictionary_file_by_file_then_rule` et par le rendu exact du projet réduit (0030), qui l'exerce à travers deux collecteurs et un verdict vrai.

**Contrat `--json`.** Comparaison structurée **du document entier**, champ par champ : `status_json_matches_the_v1_schema_field_per_field` compare le texte exact du document (format_version, summary global, les six dimensions dans l'ordre canonique, chaque `finding` sérialisé `rule`/`severity`/`summary`/`detail`/`location`). Choix confirmé : pas de golden file séparé, et pas d'assertions ciblées seules ; le document exact **est** le pin. `format_version: 1` y est littéral — mieux qu'un test dédié : le schéma complet est verrouillé d'un coup. La rigueur document est tenue autour : rien sur stdout en cas d'arrêt code 2 (`status_json_stays_strict_without_header_env_and_outside_a_project`, non colorié, sans bannière même quand `AARSHI_NO_HEADER` est vide), `summary` global sous filtre (`status_json_keeps_the_filter_to_selected_dimensions` et le test 0030 des filtres, qui figent aussi l'ordre canonique des dimensions sélectionnées, indépendamment de l'ordre des drapeaux), et `--errors-only` (`status_json_errors_only_drops_todos_but_keeps_the_counters`, les compteurs inchangés).

**Unités.** Pas de fichiers de tests unitaires dédiés : les primitives de lecture — extraction d'imports, entrées de sitemap, scan de routes, clés de dictionnaire avec doublons et `trim()` — sont tenues par les tests noirs qui les déclenchent de bout en bout (`status_scans_routes_recursively_under_the_locale_folder`, `status_reports_a_dangling_section_import`, `status_reports_duplicate_keys_in_each_dictionary`, `status_counts_whitespace_values_as_empty`…), avec l'absorption tolérante vérifiée sur dossiers et fichiers illisibles. Retenu : la couverture noire suffit ; une suite unitaire dupliquerait la maintenance sans signal neuf en v1.

**Couverture des 36 règles de [0011](0011-grilling-contrat-sortie.md).** Chaque identifiant a son ou ses test(s) noir(s) dans `tests/cli.rs` (règle, gravité, message). Matrice gelée par la passe [0030](0030-recette-transverse.md) :

| Règle | Tests noirs |
|---|---|
| `config.file-missing` | `config_reports_each_missing_base_file`, `config_reports_a_project_without_base_files_and_still_exits_one`, `recipe_a_project_carried_to_nothing_still_produces_the_full_report` |
| `config.marker-missing` | `config_reports_a_missing_pages_marker_in_the_sitemap` |
| `config.next-config-redirect` | `config_checks_the_slash_redirect_against_the_default_language` |
| `config.project-file-incomplete` | `config_reports_each_missing_key_of_an_old_format_project_file` |
| `config.option-cache-components` | `config_checks_cache_components_against_next_config` |
| `config.option-site-url` | `config_checks_the_site_url_against_lib_site` |
| `config.option-ui` | `config_checks_the_ui_option`, `config_checks_a_shadcn_ui_project` |
| `config.option-icons` | `config_checks_the_icons_option` |
| `config.import-broken` | `config_reports_a_broken_import_in_a_base_file`, `config_leaves_missing_base_files_to_the_file_missing_rule` |
| `i18n.languages-mismatch` | `status_reports_a_languages_mismatch`, `status_reports_a_missing_config_file_without_stopping_the_report`, `recipe_a_missing_i18n_file_is_reported_once_by_i18n` |
| `i18n.default-locale-mismatch` | `status_reports_a_default_locale_mismatch`, `status_reports_a_missing_default_locale` |
| `i18n.direction-mismatch` | `status_reports_an_rtl_language_declared_ltr`, `status_reports_a_direction_entry_missing_for_a_language` |
| `i18n.index-mismatch` | `status_checks_the_imports_of_index_ts_per_language`, `status_reports_an_index_import_of_an_undeclared_language`, `status_reports_a_message_key_taken_from_the_wrong_language`, `status_reports_a_missing_index_file_without_stopping_the_report` |
| `i18n.dictionary-missing` | `status_reproduces_the_contract_render_on_a_fixture`, `status_json_matches_the_v1_schema_field_per_field`, `recipe_a_project_carried_to_nothing_still_produces_the_full_report` |
| `i18n.key-missing` | `status_reports_a_key_missing_from_a_non_default_dictionary` |
| `i18n.key-duplicate` | `status_reports_duplicate_keys_in_each_dictionary` |
| `i18n.key-empty` | `status_counts_whitespace_values_as_empty`, `status_todos_alone_still_exit_zero`, `status_reports_empty_translations_until_they_are_filled` |
| `i18n.key-extra` | `status_reports_a_key_extra_in_a_non_default_dictionary` |
| `i18n.dictionary-orphan` | `status_reports_a_dictionary_orphan` |
| `pages.route-missing` | `status_reports_a_view_without_its_route` |
| `pages.route-orphan` | `status_reports_a_hand_added_route_without_a_view`, `status_scans_routes_recursively_under_the_locale_folder` |
| `pages.title-key-missing` | `status_reports_a_title_key_missing_from_a_dictionary` |
| `pages.sitemap-entry-missing` | `status_reports_a_missing_sitemap_entry` |
| `pages.sitemap-entry-orphan` | `status_reports_an_orphan_sitemap_entry` |
| `pages.view-import-missing` | `status_reports_a_missing_view_import` |
| `pages.import-broken` | `status_reports_a_broken_import_in_a_view` |
| `sections.file-missing` | `status_reports_a_dangling_section_import` |
| `sections.title-key-missing` | `status_reports_a_section_title_key_missing_from_a_dictionary` |
| `sections.marker-missing` | `status_reports_the_missing_imports_marker`, `status_reports_the_missing_sections_marker`, `status_reports_no_marker_finding_for_a_complete_view` |
| `sections.orphan` | `status_reports_an_orphan_section_as_a_todo` |
| `sections.import-broken` | `status_reports_a_broken_import_in_a_section` |
| `components.icon-unconverted` | `components_report_unconverted_icons`, `components_ignore_a_tsx_without_svg` |
| `components.import-broken` | `components_report_a_broken_import_in_a_custom_component` |
| `dependencies.package-missing` | `status_reports_an_undeclared_npm_package`, `status_reports_a_scoped_package_by_its_root_name`, `status_accepts_a_package_declared_in_dev_dependencies`, `status_treats_a_bare_import_as_a_package_not_a_local_file`, `status_parks_a_package_import_in_dependencies_not_in_the_files_dimension`, `status_groups_the_importers_of_one_undeclared_package` |
| `dependencies.lockfile-mismatch` | `status_checks_the_lockfile_against_the_package_manager` |
| `dependencies.modules-missing` | `status_reports_missing_node_modules_as_a_todo` |

**Invariants transverses du contrat de sortie.** Chacun est tenu par un test : le tri (ci-dessus, cumulé dans les rendus exacts) ; la règle sans arrêt précoce — `config_reports_a_project_without_base_files_and_still_exits_one`, `status_reports_no_pages_finding_without_views_or_routes`, `status_absorbs_an_unreadable_package_json`, et le pire cas 0030 (projet réduit au seul `aarshi.toml`, les six dimensions visitées, code de sortie vrai) ; `--dimension` répétable + virgules — forme et ligne `info: N findings hidden by --dimension` (`status_human_output_reports_hidden_findings_under_a_filter`, qui fige exactement la phrase), code de sortie et `summary` intacts sous filtre (le même test, `status_json_keeps_the_filter_to_selected_dimensions`, et `recipe_dimension_repeats_and_lists_keep_the_code_and_the_summary`) ; `--errors-only` — les todos quittent le rendu humain (dimension qui ne porte que des todos marquée nue) et les `findings` JSON, sans mouvement de code ni de compteurs, **sans jamais** déclencher la ligne info (`status_json_errors_only_drops_todos_but_keeps_the_counters`, `recipe_errors_only_hides_the_todos_without_the_info_line`) ; la stricte rigueur de `--json` et le silence du stdout d'un arrêt, déjà notés. La règle du `to do` à forme fixe (`1 to do`, `2 to do`), le pluriel de `amount`, et la ligne d'info réservée au filtre de dimension sont, ensemble, figés par les deux tests du bloc 0030.

**Un constat par objet** — réconciliation du doublon consigné dans [0025](0025-dimension-i18n.md) (0027 a déjà figé la continuité : l'import pendant d'une vue remonte à `sections.file-missing`, pas à `pages`). Décision : la règle de rattachement de [0013](0013-grilling-dimensions.md) l'emporte — un constat appartient à **exactement une** dimension, celle de l'objet qu'il concerne.

- Fichiers manquants ou illisibles dont le contenu appartient à une dimension de contenu (`lib/i18n/config.ts`, `lib/i18n/index.ts`) : un seul constat, **par celle-ci** — `i18n.languages-mismatch` ou `i18n.index-mismatch` (« is missing or unreadable », libellé retenu par 0025). `config.file-missing` se tait pour ces deux fichiers (correctif mineur `src/status/config.rs`, constante `SELF_REPORTED` — additionnel, venu de 0030) : l'inventaire de [0008](0008-grilling-invariants-config.md) reste la source des fichiers attendus (les chemins y restent en liste pour absorber les imports d'un fichier parti sans constat nouveau), la classification appartient à 0013.
- `app/sitemap.ts` : aucun constat de la part des contrôles d'entrée de `pages` (0016, addendum) — un seul constat, par l'inventaire `config.file-missing`, puisque chaque règle de `pages` vouée à la sitemap parle des entrées et non de la présence du fichier ; la comparaison sautée tient. Gelé par `recipe_a_missing_sitemap_is_reported_once_by_config`.
- Les trois tests qui comptaient le doublon sont réécrits (`status_reports_a_missing_index_file_without_stopping_the_report` : un seul constat, `config_reports_a_project_without_base_files_and_still_exits_one` : « missing » compté sur l'inventaire se solde par 11, `config_leaves_missing_base_files_to_the_file_missing_rule` : le pin d'absorption des imports reste tenu sur `lib/site.ts`, fichier de dimension `config`).

Les deux mises en attente d'[0028](0028-dimension-components.md) restent respectées (aucun élargissement du périmètre) : les imports des fichiers connus restent classés par dimension de l'objet fautif (`pages` pour vues et routes, `sections` pour les sections, `config` pour les fichiers de base) sans que `components` n'en ré-ouvre la promenade, et rien n'a été fait pour les composants `custom` non utilisés. Aucune règle supprimée, aucune gravité changée : chaque objet de l'inventaire 0008 garde son test d'existence, tout juste déplace-t-il de dimension quand il appartient de droit au contenu d'une autre.
