---
labels: [wayfinder:grilling]
parent: 0005-map-status.md
blocked-by: []
status: closed
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

**Catalogue pages/routes/sections/composants retenu.** L'attente des clés vient du système de fichiers (voir la note au-dessus).

**Pages et routes** — tout en `error` :

- vue présente et route absente ;
- route présente dans `app/[locale]/` et vue absente — **route orpheline** (aujourd'hui invisible) ;
- clé `"<page>.title"` absente d'un dictionnaire ;
- page absente de `app/sitemap.ts` ;
- entrée sitemap sans page correspondante (`/` = home est exclu : il est codé en dur dans le template) ;
- import de la vue (`@/views/<page>/<page>.view`) absent ou incorrect dans la route.

**Sections (`views/<page>/sections/<section>.tsx`)** :

| Constat | Gravité |
|---|---|
| Import pendant (vue → `sections/<section>.tsx` absent) | `error` |
| Clé `"<page>.<section>.title"` absente d'un dictionnaire | `error` |
| Marqueur `// aarshi:imports:end` ou `{/* aarshi:sections:end */}` absent de la vue | `error` |
| Section orpheline (`sections/*.tsx` jamais importée) | `todo` |

**Icônes** : `components/icons/<name>.svg` sans `<name>.tsx` → `todo` (conversion pas encore faite) ; `<name>.tsx` sans `.svg` → **ignoré** (l'SVG a été retiré après conversion).

**Liens et imports** : `status` vérifie que les imports des fichiers qu'il connaît (routes, vues, sections, `app/[locale]/layout.tsx`) **résolvent vers un fichier existant** → `error`. Un composant de `components/custom/` **non utilisé n'est pas signalé** (ce n'est pas un invariant). La *présence* de `components/language-switcher.tsx` relève du ticket « Invariants config/projet et outillage » ; son *import* est couvert ici.

**Non vérifié, et pourquoi** : une clé présente dans le dictionnaire par défaut mais sans page ni section correspondante n'est **pas** signalée — le dev peut ajouter des clés à la main (`common.cta`), donc l'absence de correspondance fichier n'est pas une anomalie.

**Règles de mise en œuvre** :

- Le scan des routes est **récursif** sous `app/[locale]/` (toute `page.tsx`).
- **Règle transverse** (notée dans les Notes de la carte) : une attente non satisfaite — fichier/dossier clé absent (`views/`, `lib/i18n/config.ts`, dictionnaire de la langue par défaut, …) — est un **constat** (`error`), jamais un arrêt : `status` continue et rapporte tout ce qu'il peut.
