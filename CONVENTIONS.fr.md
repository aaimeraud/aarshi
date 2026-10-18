# Conventions

Conventions pour le dépôt `aarshi` et pour les projets qu'il génère. Version anglaise : [`CONVENTIONS.md`](CONVENTIONS.md).

## Dépôt

- Langue : documentation en anglais (`README.md`, `CONVENTIONS.md`) avec une version française (`*.fr.md`). Code et commentaires de code en anglais.
- Les commentaires sont courts et uniquement là où le comportement n'est pas évident. Les fonctions Rust publiques ont un commentaire `///`.
- Stack : Rust, `clap` pour les commandes, `inquire` pour les questions interactives, un binaire unique.
- Le travail de spécification est suivi en issues numérotées dans `issues/` (`NNNN-slug.md`), avec un frontmatter (`labels`, `status`, `parent`, `blocked-by`). La carte est `issues/0001-map.md`.
- Les messages de commit suivent `type(scope): summary`.

## Projets générés

- TypeScript, Next.js App Router, Tailwind, ESLint, sans dossier `src/`.
- `AGENTS.md` est généré à la racine du projet.
- La télémétrie est désactivée par projet (`.env`), pas globalement.
- Structure : `views/` pour les vues de page, `sections/` pour les sections, une arborescence de routes commune sous `app/[locale]/`, dictionnaires typés par langue.
- Le sens d'écriture (`ltr` ou `rtl`) vient de la langue, pas de la page.
- Composants : shadcn (stylisé, basé sur Base UI) par défaut, ou Base UI directement. Les classes passent par `cn` de `@sglara/cn`. `components/custom/` contient les composants du projet, `components/icons/` les SVG convertis par `aarshi icons`.
- Le TypeScript généré utilise des fonctions fléchées et des exports nommés pour les sections, les icônes et les composants personnalisés.

### Invariants du projet généré

Voici les invariants qu'un projet généré doit respecter ; `aarshi status` les vérifie. Chaque invariant est une règle avec un identifiant stable (`<dimension>.<sujet>-<problème>`), regroupée ci-dessous par dimension, avec sa gravité entre parenthèses : `error` (le projet est cassé) ou `todo` (quelque chose reste à faire).

#### config

- `config.file-missing` (`error`) : un fichier ou dossier de base est absent (`app/[locale]/layout.tsx`, `app/sitemap.ts`, `app/robots.ts`, `lib/site.ts`, `lib/i18n/config.ts`, `lib/i18n/index.ts`, `components/language-switcher.tsx`, `tsconfig.json`, `eslint.config.mjs`, `postcss.config.mjs`, `next.config.ts`, `components/custom/`, `components/icons/`).
- `config.marker-missing` (`error`) : le marqueur `// aarshi:pages:end` est absent de `app/sitemap.ts`.
- `config.next-config-redirect` (`error`) : la redirection `"/"` de `next.config.ts` est incohérente avec `languages[0]`.
- `config.project-file-incomplete` (`todo`) : une clé persistée est absente d'`aarshi.toml`.
- `config.option-cache-components` (`error`) : `cache_components` et `cacheComponents` divergent.
- `config.option-site-url` (`error`) : `site_url` et `SITE_URL` de `lib/site.ts` divergent.
- `config.option-ui` (`error`) : `ui` et les dépendances et composants d'UI divergent.
- `config.option-icons` (`error`) : `icons` et `lucide-react` divergent.
- `config.import-broken` (`error`) : un import cassé dans un fichier de base.

#### i18n

- `i18n.languages-mismatch` (`error`) : les `languages` de `config.ts` et d'`aarshi.toml` divergent.
- `i18n.default-locale-mismatch` (`error`) : `defaultLocale` diffère de `languages[0]`.
- `i18n.direction-mismatch` (`error`) : la table `direction` est incomplète ou incorrecte.
- `i18n.index-mismatch` (`error`) : `lib/i18n/index.ts` est désynchronisé.
- `i18n.dictionary-missing` (`error`) : un dictionnaire est absent pour une langue déclarée.
- `i18n.key-missing` (`error`) : une clé est absente par rapport au dictionnaire par défaut.
- `i18n.key-duplicate` (`error`) : une clé dupliquée dans un dictionnaire.
- `i18n.key-empty` (`todo`) : une valeur est vide après `trim()`.
- `i18n.key-extra` (`todo`) : une clé est absente du dictionnaire par défaut.
- `i18n.dictionary-orphan` (`todo`) : un dictionnaire existe pour une langue hors `aarshi.toml`.

#### pages

- `pages.route-missing` (`error`) : une vue sans route.
- `pages.route-orphan` (`error`) : une route sans vue.
- `pages.title-key-missing` (`error`) : la clé `"<page>.title"` est absente d'un dictionnaire.
- `pages.sitemap-entry-missing` (`error`) : une page est absente de `app/sitemap.ts`.
- `pages.sitemap-entry-orphan` (`error`) : une entrée de sitemap sans page.
- `pages.view-import-missing` (`error`) : l'import de la vue est absent ou incorrect dans la route.
- `pages.import-broken` (`error`) : un import cassé dans une vue ou une route.

#### sections

- `sections.file-missing` (`error`) : un import pendant (vue → `sections/<section>.tsx` absente).
- `sections.title-key-missing` (`error`) : la clé `"<page>.<section>.title"` est absente d'un dictionnaire.
- `sections.marker-missing` (`error`) : le marqueur `// aarshi:imports:end` ou `{/* aarshi:sections:end */}` est absent de la vue.
- `sections.orphan` (`todo`) : une section jamais importée.
- `sections.import-broken` (`error`) : un import cassé dans une section.

#### components

- `components.icon-unconverted` (`todo`) : un `.svg` sans composant généré.
- `components.import-broken` (`error`) : un import cassé dans `components/`.

#### dependencies

- `dependencies.package-missing` (`error`) : un paquet importé non déclaré dans `package.json` (`@sglara/cn`, `next`, `react`, `@base-ui/react`, `lucide-react`, ...).
- `dependencies.lockfile-mismatch` (`error`) : un lockfile incohérent avec `package_manager`.
- `dependencies.modules-missing` (`todo`) : `node_modules/` est absent.

## Non spécifié

- Nommage exact des templates et des fichiers générés (voir la carte).

