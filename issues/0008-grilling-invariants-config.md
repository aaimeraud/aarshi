---
labels: [wayfinder:grilling]
parent: 0005-map-status.md
blocked-by: []
status: open
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

<!-- à remplir à la résolution : invariants retenus, règle exacte, gravité error/todo, exemples -->
