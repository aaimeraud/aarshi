---
labels: [wayfinder:grilling]
parent: 0005-map-status.md
blocked-by: []
status: closed
assignee: aaimeraud
---

# Invariants config/projet et outillage : lesquels `status` vérifie-t-il ?

## Question

Parmi les invariants de configuration et d'outillage candidats ci-dessous, lesquels `status` vérifie-t-il en v1, avec quelle règle exacte, et avec quel classement `error` (conformité cassée) / `todo` (reste-à-faire attendu) ?

Candidats relevés dans `src/setup.rs`, `src/config.rs`, `src/project.rs` :

**Config et fichiers du projet** :

- `aarshi.toml` présent et valide (schéma strict `deny_unknown_fields`, clés `languages` + `package_manager`).
- `next.config.ts` : redirect `"/"` → `"/<langue par défaut>"` cohérent avec `languages[0]` (aujourd'hui **non vérifié**).
- Option `cacheComponents` : non persistée aujourd'hui (`aarshi.toml` ne stocke que `languages` et `package_manager`), donc invérifiable. Décider : faut-il la persister pour pouvoir la contrôler, ou l'écarter du catalogue ?
- `.env` : ligne `NEXT_TELEMETRY_DISABLED=1` présente.
- Fichiers de base attendus présents : `app/[locale]/layout.tsx`, `app/sitemap.ts`, `app/robots.ts`, `lib/site.ts`, `lib/i18n/{config,index}.ts`, `components/language-switcher.tsx`, `components/custom/.gitkeep`, `components/icons/.gitkeep`.
- Marqueur `// aarshi:pages:end` présent dans `app/sitemap.ts`.

**Dépendances et outillage** :

- `node_modules` installé.
- Dépendances déclarées cohérentes avec l'état : `@sglara/cn` toujours présent ; `lucide-react` / `@base-ui/react` selon les choix d'UI/icônes (ces choix **ne sont pas persistés** dans `aarshi.toml` — décider quoi faire).
- Lockfile ↔ `package_manager` de `aarshi.toml` (par ex. `pnpm-lock.yaml` avec `pnpm`).
- Fichiers de config présents : `tsconfig.json`, `eslint.config.mjs`, `postcss.config.mjs`, `next.config.ts`.

## Resolution

**Catalogue config/projet et outillage retenu.**

**`aarshi.toml` étendu** (nouvelle décision) : en plus de `languages` et `package_manager`, `setup` écrit désormais `ui` (`shadcn` ou `base-ui`), `icons` (`lucide` ou `none`), `cache_components` (booléen) et `site_url` (chaîne) — les valeurs littérales actuelles sont conservées. `agent_feedback` **n'est pas** persisté : c'est une préférence d'outillage d'agent, pas un état du site. Les nouvelles clés sont validées comme les existantes (ensembles connus). Conséquence d'implémentation : `add-lang`, qui réécrit `aarshi.toml` en entier, doit **préserver** ces clés.

**Fichiers et liens** — tout en `error` :

- `aarshi.toml` présent et valide (schéma strict) ;
- `next.config.ts` présent, redirect `"/"` → `"/<langue par défaut>"` cohérent avec `languages[0]` ;
- fichiers attendus présents : `app/[locale]/layout.tsx`, `app/sitemap.ts`, `app/robots.ts`, `lib/site.ts`, `lib/i18n/config.ts`, `lib/i18n/index.ts`, `components/language-switcher.tsx`, `tsconfig.json`, `eslint.config.mjs`, `postcss.config.mjs` ;
- dossiers `components/custom/` et `components/icons/` présents — `.gitkeep` **non exigé** (il disparaît légitimement dès qu'un fichier existe) ;
- marqueur `// aarshi:pages:end` présent dans `app/sitemap.ts`.

**Options persistées** — contrôlées contre le projet, `error` si écart :

- `cache_components` ↔ `cacheComponents` dans `next.config.ts` ;
- `site_url` ↔ `SITE_URL` dans `lib/site.ts` ;
- `ui` : `base-ui` → `@base-ui/react` déclaré **et** composants Base UI présents ; `shadcn` → composants shadcn présents (`components/ui/button.tsx`, `button-group.tsx`) ;
- `icons` : `lucide` → `lucide-react` déclaré ; `none` → aucune exigence.

**Clé absente d'`aarshi.toml`** (projet généré avant l'ajout des clés) : `todo` — le fichier de config est à compléter ; ce n'est pas `error`, et ce n'est pas non plus une vérification silencieusement sautée.

**Outillage et dépendances** :

| Constat | Gravité |
|---|---|
| `@sglara/cn` non déclaré dans `package.json` | `error` |
| Import de paquet non déclaré (`next`, `react`, `@base-ui/react`, `lucide-react`, …) dans les fichiers connus | `error` |
| Lockfile incohérent avec `package_manager` (`pnpm-lock.yaml` avec `npm`, …) | `error` |
| `node_modules` absent | `todo` |

**Non vérifié** : la ligne `NEXT_TELEMETRY_DISABLED=1` de `.env` ne fait pas partie du catalogue.
