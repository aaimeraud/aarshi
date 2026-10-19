# Conventions des projets générés

Ces conventions décrivent les projets Next.js créés par `aarshi setup` et étendus par `aarshi add`. Elles s'adressent aux développeurs de ces sites. Les conventions pour contribuer à aarshi lui-même sont dans [CONTRIBUTING.fr.md](CONTRIBUTING.fr.md). Version anglaise : [CONVENTIONS.md](CONVENTIONS.md).

## Structure du projet

- TypeScript, Next.js App Router, Tailwind, ESLint, sans dossier `src/`.
- `AGENTS.md` est généré à la racine du projet.
- La télémétrie est désactivée par projet (`.env`), pas globalement.
- `views/` contient les vues de page et `views/<page>/sections/` les sections, avec une arborescence de routes commune sous `app/[locale]/` et des dictionnaires typés par langue.
- Le sens d'écriture (`ltr` ou `rtl`) vient de la langue, pas de la page.
- Composants : shadcn (stylisé, basé sur Base UI) par défaut, ou Base UI directement. Les classes passent par `cn` de `@sglara/cn`. `components/custom/` contient les composants du projet et `components/icons/` les SVG convertis par `aarshi icons`.
- Le TypeScript généré utilise des fonctions fléchées et des exports nommés pour les sections, les icônes et les composants personnalisés.

## Invariants vérifiés par `aarshi status`

Ce sont les invariants qu'un projet généré doit respecter. Chacun est une règle avec un identifiant stable (`<dimension>.<sujet>-<problème>`), regroupée ci-dessous par dimension, avec sa gravité entre parenthèses : `error` (le projet est cassé) ou `todo` (quelque chose reste à faire).

### config

- `config.file-missing` (`error`) : un fichier ou dossier de base manque (`app/[locale]/layout.tsx`, `app/sitemap.ts`, `app/robots.ts`, `lib/site.ts`, `lib/i18n/config.ts`, `lib/i18n/index.ts`, `components/language-switcher.tsx`, `tsconfig.json`, `eslint.config.mjs`, `next.config.ts`, `components/custom/`, `components/icons/`).
- `config.tailwind-missing` (`error`) : Tailwind n'est branché ni par `@tailwindcss/turbopack` dans `next.config.ts`, ni par un `postcss.config.*` qui utilise `@tailwindcss/postcss`.
- `config.marker-missing` (`error`) : le marqueur `// aarshi:pages:end` manque dans `app/sitemap.ts`.
- `config.next-config-redirect` (`error`) : la redirection `"/"` de `next.config.ts` est incohérente avec `languages[0]`.
- `config.project-file-incomplete` (`todo`) : une clé persistée manque dans `aarshi.toml`.
- `config.option-cache-components` (`error`) : `cache_components` et `cacheComponents` se contredisent.
- `config.option-site-url` (`error`) : `site_url` et `SITE_URL` de `lib/site.ts` se contredisent.
- `config.option-ui` (`error`) : `ui` et les dépendances et composants d'interface se contredisent.
- `config.option-icons` (`error`) : `icons` et `lucide-react` se contredisent.
- `config.import-broken` (`error`) : un import cassé dans un fichier de base.

### i18n

- `i18n.languages-mismatch` (`error`) : les `languages` de `config.ts` et de `aarshi.toml` se contredisent.
- `i18n.default-locale-mismatch` (`error`) : `defaultLocale` diffère de `languages[0]`.
- `i18n.direction-mismatch` (`error`) : la table `direction` est incomplète ou fausse.
- `i18n.index-mismatch` (`error`) : `lib/i18n/index.ts` est désynchronisé.
- `i18n.dictionary-missing` (`error`) : un dictionnaire manque pour une langue déclarée.
- `i18n.key-missing` (`error`) : une clé manque par rapport au dictionnaire par défaut.
- `i18n.key-duplicate` (`error`) : une clé est en double dans un dictionnaire.
- `i18n.key-empty` (`todo`) : une valeur est vide après `trim()`.
- `i18n.key-extra` (`todo`) : une clé est absente du dictionnaire par défaut.
- `i18n.dictionary-orphan` (`todo`) : un dictionnaire existe pour une langue hors de `aarshi.toml`.

### pages

- `pages.route-missing` (`error`) : une vue sans route.
- `pages.route-orphan` (`error`) : une route sans vue.
- `pages.title-key-missing` (`error`) : la clé `"<page>.title"` manque dans un dictionnaire.
- `pages.sitemap-entry-missing` (`error`) : une page manque dans `app/sitemap.ts`.
- `pages.sitemap-entry-orphan` (`error`) : une entrée de plan du site sans page.
- `pages.view-import-missing` (`error`) : l'import de la vue manque ou est incorrect dans la route.
- `pages.import-broken` (`error`) : un import cassé dans une vue ou une route.

### sections

- `sections.file-missing` (`error`) : un import pendant (vue → `sections/<section>.tsx` manquant).
- `sections.title-key-missing` (`error`) : la clé `"<page>.<section>.title"` manque dans un dictionnaire.
- `sections.marker-missing` (`error`) : le marqueur `// aarshi:imports:end` ou `{/* aarshi:sections:end */}` manque dans la vue.
- `sections.orphan` (`todo`) : une section jamais importée.
- `sections.import-broken` (`error`) : un import cassé dans une section.

### components

- `components.icon-unconverted` (`todo`) : un `.svg` sans composant généré.
- `components.import-broken` (`error`) : un import cassé dans `components/`.

### dependencies

- `dependencies.package-missing` (`error`) : un paquet importé non déclaré dans `package.json` (`@sglara/cn`, `next`, `react`, `@base-ui/react`, `lucide-react`, ...).
- `dependencies.lockfile-mismatch` (`error`) : un lockfile incohérent avec `package_manager`.
- `dependencies.modules-missing` (`todo`) : `node_modules/` est absent.
