---
labels: [wayfinder:grilling]
parent: 0005-map-status.md
blocked-by: []
status: open
assignee: aaimeraud
---

# Invariants pages, routes, sections et composants : lesquels `status` vérifie-t-il ?

## Question

Parmi les invariants pages/routes/sections/composants candidats ci-dessous, lesquels `status` vérifie-t-il en v1, avec quelle règle exacte, et avec quel classement `error` (conformité cassée) / `todo` (reste-à-faire attendu) ?

Candidats relevés dans `src/add.rs`, `src/templates.rs`, `src/dictionary.rs` :

**Pages et routes** :

- `views/<page>/<page>.view.tsx` ↔ `app/[locale]/<page>/page.tsx` : `check` ne vérifie que le sens vue → route ; le sens route → vue (route orpheline ajoutée à la main) n'est pas couvert.
- Clé de dictionnaire `"<page>.title"` présente dans chaque dictionnaire.
- Entrée sitemap `"/<page>"` dans `app/sitemap.ts` ↔ page (`"/"` pour `home`) — aujourd'hui **non vérifié**.
- Import de la vue `@/views/<page>/<page>.view` présent et correct dans la route.

**Sections (`views/<page>/sections/<section>.tsx`)** :

- Section référencée par une vue (`import` + `<XSection … />`) ↔ fichier `sections/<section>.tsx` existant (aujourd'hui invisible : import pendant ou section orpheline non détectés).
- Clé de dictionnaire `"<page>.<section>.title"` présente dans chaque dictionnaire.
- Marqueurs `// aarshi:imports:end` et `{/* aarshi:sections:end */}` présents dans la vue (sans eux, `add -s` échoue à l'exécution, mais `check` passe).

**Composants et icônes** :

- `components/custom/<name>.tsx` référencés par les vues/composants.
- `components/icons/<name>.svg` ↔ `components/icons/<name>.tsx` (icône `.svg` non convertie, ou `.tsx` sans `.svg`).
- `components/language-switcher.tsx` présent et importé par `app/[locale]/layout.tsx`.

**Source unique de l'attente des clés** (décision du ticket « Invariants i18n… ») : la clé attendue d'une page ou d'une section se déduit de la **structure de fichiers** (`views/<page>/…` → `"<page>.title"`, `views/<page>/sections/<section>.tsx` → `"<page>.<section>.title"`) ; ce ticket ne parse pas les vues. C'est ici que le cas « clé référencée mais absente de tous les dictionnaires » est traité, pas dans le catalogue i18n.

## Resolution

<!-- à remplir à la résolution : invariants retenus, règle exacte, gravité error/todo, exemples -->
