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

**L'extension d'`aarshi.toml` fait partie de cette carte, en tranche amont dédiée.** Faute d'écriture de `ui`, `icons`, `cache_components` et `site_url`, `status` n'aurait rien à vérifier pour les options persistées : la tranche est extraite de la carte en un ticket séparé placé avant les tickets `status`, [0022-amont-options-config](0022-amont-options-config.md). Livrée par lui :

- `setup` écrit désormais les six clés (`languages`, `package_manager`, `ui`, `icons`, `cache_components`, `site_url`) via `Config::to_project_file()` ;
- `add-lang`, qui réécrit `aarshi.toml` en entier, préserve ces clés ;
- le schéma strict (`deny_unknown_fields`) étendu avec les nouvelles clés reste tolérant à leur absence : un fichier à l'ancien format (`languages` seul suffit, `package_manager` étant déjà optionnel) parse sans erreur, et les clés inconnues restent rejetées ; les valeurs sont validées comme les existantes (`ui` ∈ shadcn/base-ui, `icons` ∈ lucide/none, `site_url` en http(s)) ;
- conséquence convergente : une fois la tranche amont livrée, `config.project-file-incomplete` redeviendra un `todo` exceptionnel (projet généré avant l'extension) et non l'état attendu d'un projet neuf ;
- `agent_feedback` n'est pas persisté, conformément à [0008](0008-grilling-invariants-config.md) (préférence d'outillage d'agent, pas un état du site).

**La disparition de `Command::Check` est hors périmètre de cette carte.** La plomberie et la migration des tests `check` restent instruites par le ticket [0018-grilling-codes-sortie-alias-check](0018-grilling-codes-sortie-alias-check.md) ; la carte `status` s'appuie sur sa résolution.

**Aucun autre travail amont : l'inventaire des fichiers de base est complet.** Vérification faite contre `src/setup.rs` (`generate()` et `create_component_folders()`) et `src/templates.rs` : `setup` génère bien tous les fichiers attendus au catalogue de [0008](0008-grilling-invariants-config.md) — `app/[locale]/layout.tsx`, `app/sitemap.ts`, `app/robots.ts`, `lib/site.ts`, `lib/i18n/config.ts`, `lib/i18n/index.ts`, `lib/i18n/dictionaries/<locale>.ts` (un par langue), `components/language-switcher.tsx`, `components/ui/button.tsx` et `button-group.tsx`, les dossiers `components/custom/` et `components/icons/` — et les marqueurs (`aarshi:pages:end`, `aarshi:messages:end`, `aarshi:imports:end`, `aarshi:sections:end`) sont posés par les gabarits. Aucun fichier de base manquant, aucun marqueur à ajouter.
