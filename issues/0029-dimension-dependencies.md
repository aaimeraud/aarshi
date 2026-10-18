---
labels: [wayfinder:implement]
parent: 0014-map-implementation-status.md
blocked-by: [0026-dimension-pages.md]
status: open
---

# Dimension `dependencies` : paquets déclarés, lockfile, `node_modules`

## Tranche

Remplir la dimension `dependencies` (catalogue [0008](0008-grilling-invariants-config.md), identifiants [0011](0011-grilling-contrat-sortie.md)) :

| Règle | Gravité | Constat |
|---|---|---|
| `dependencies.package-missing` | `error` | paquet importé non déclaré dans `package.json` (`next`, `react`, `@sglara/cn`, `@base-ui/react`, `lucide-react`, …), dans les fichiers connus |
| `dependencies.lockfile-mismatch` | `error` | lockfile incohérent avec `package_manager` (`pnpm-lock.yaml` avec `npm`, …) |
| `dependencies.modules-missing` | `todo` | `node_modules/` absent |

Rappels de spec : un **import de paquet npm non déclaré** tombe toujours dans `dependencies` (l'objet concerné est la dépendance, pas le fichier) ; le scan des packages porte sur les fichiers connus du collecteur — sa couverture s'élargit au fil des tranches précédentes ; la ligne de télémétrie du `.env` ne fait pas partie du catalogue.

## Critères d'acceptance

- [ ] Chaque règle a au moins un test noir (constat déclenché sur fixture, `rule`, gravité, message).
- [ ] Un import `lucide-react` sans dépendance déclarée produit bien un constat `dependencies` même quand l'import se trouve dans un fichier des dimensions précédentes.
- [ ] Chaque gestionnaire (`npm`, `pnpm`, `yarn`, `bun`) a son cas nominal et son lockfile incohérent.
- [ ] `node_modules/` absent → `todo` unique, code de sortie inchangé.
