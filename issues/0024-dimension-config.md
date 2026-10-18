---
labels: [wayfinder:implement]
parent: 0014-map-implementation-status.md
blocked-by: [0022-amont-options-config.md, 0023-socle-tracer-i18n.md]
status: open
---

# Dimension `config` : fichiers, marqueurs, redirect, options contrôlées

## Tranche

Remplir la dimension `config` (catalogue [0008](0008-grilling-invariants-config.md), identifiants [0011](0011-grilling-contrat-sortie.md)) : `config.file-missing` (fichiers et dossiers de base), `config.marker-missing` (`// aarshi:pages:end` de `app/sitemap.ts`), `config.next-config-redirect` (redirect `"/"` cohérent avec `languages[0]`), `config.project-file-incomplete` (clé persistée absente d'`aarshi.toml`, `todo` — grâce à l'amont [0022](0022-amont-options-config.md) les nouveaux projets sont complets), `config.option-cache-components`, `config.option-site-url`, `config.option-ui` (base-ui → dépendance **et** composants ; shadcn → composants), `config.option-icons` (lucide → `lucide-react`), `config.import-broken` (import cassé dans un fichier de base, p. ex. `app/[locale]/layout.tsx`).

Rappels de spec : un fichier-clé absent est un **constat**, jamais un arrêt ; la ligne `NEXT_TELEMETRY_DISABLED=1` du `.env` n'est **pas** vérifiée ; `agent_feedback` n'est pas persisté.

## Critères d'acceptance

- [ ] Chaque règle a au moins un test noir qui en déclenche le constat (`rule`, gravité, message) sur une fixture.
- [ ] Un projet dépourvu de fichiers de base produit une constellation de constats `error` et se termine quand même (`0`/`1` selon les `error`), sans plantage.
- [ ] Un `aarshi.toml` à l'ancien format produit exactement quatre `config.project-file-incomplete` (`todo`) et la sortie reste code `0`.
- [ ] Les composants `ui`/`icons` (base-ui, shadcn, lucide, none) sont contrôlés chacun dans leur cas nominal et leur cas d'écart.
