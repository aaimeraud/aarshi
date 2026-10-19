# Conventions of the generated projects

These conventions describe the Next.js projects created by `aarshi setup` and extended by `aarshi add`. They are written for the developers of those sites. Conventions for contributing to aarshi itself are in [CONTRIBUTING.md](CONTRIBUTING.md). French version: [CONVENTIONS.fr.md](CONVENTIONS.fr.md).

## Project structure

- TypeScript, Next.js App Router, Tailwind, ESLint, no `src/` folder.
- `AGENTS.md` is generated at the project root.
- Telemetry is disabled per project (`.env`), not globally.
- `views/` holds page views and `views/<page>/sections/` the page sections, with one shared route tree under `app/[locale]/` and typed dictionaries per language.
- Text direction (`ltr` or `rtl`) comes from the language, not from the page.
- Components: shadcn (styled, Base UI based) by default, or Base UI directly. Class names go through `cn` from `@sglara/cn`. `components/custom/` holds project components and `components/icons/` the SVGs converted by `aarshi icons`.
- Generated TypeScript uses arrow functions and named exports for sections, icons and custom components.

## Invariants checked by `aarshi status`

These are the invariants a generated project must satisfy. Each is one rule with a stable identifier (`<dimension>.<subject>-<problem>`), grouped below by dimension, with its severity in parentheses: `error` (the project is broken) or `todo` (something is still to be done).

### config

- `config.file-missing` (`error`): a base file or folder is missing (`app/[locale]/layout.tsx`, `app/sitemap.ts`, `app/robots.ts`, `lib/site.ts`, `lib/i18n/config.ts`, `lib/i18n/index.ts`, `components/language-switcher.tsx`, `tsconfig.json`, `eslint.config.mjs`, `next.config.ts`, `components/custom/`, `components/icons/`).
- `config.tailwind-missing` (`error`): Tailwind is wired neither by `@tailwindcss/turbopack` in `next.config.ts` nor by a `postcss.config.*` that uses `@tailwindcss/postcss`.
- `config.marker-missing` (`error`): the `// aarshi:pages:end` marker is missing from `app/sitemap.ts`.
- `config.next-config-redirect` (`error`): the `"/"` redirect of `next.config.ts` is inconsistent with `languages[0]`.
- `config.project-file-incomplete` (`todo`): a persisted key is missing from `aarshi.toml`.
- `config.option-cache-components` (`error`): `cache_components` and `cacheComponents` disagree.
- `config.option-site-url` (`error`): `site_url` and `SITE_URL` of `lib/site.ts` disagree.
- `config.option-ui` (`error`): `ui` and the UI dependencies and components disagree.
- `config.option-icons` (`error`): `icons` and `lucide-react` disagree.
- `config.import-broken` (`error`): a broken import in a base file.

### i18n

- `i18n.languages-mismatch` (`error`): the `languages` of `config.ts` and `aarshi.toml` disagree.
- `i18n.default-locale-mismatch` (`error`): `defaultLocale` differs from `languages[0]`.
- `i18n.direction-mismatch` (`error`): the `direction` table is incomplete or wrong.
- `i18n.index-mismatch` (`error`): `lib/i18n/index.ts` is out of sync.
- `i18n.dictionary-missing` (`error`): a dictionary is missing for a declared language.
- `i18n.key-missing` (`error`): a key is missing compared to the default dictionary.
- `i18n.key-duplicate` (`error`): a duplicate key in a dictionary.
- `i18n.key-empty` (`todo`): a value is empty after `trim()`.
- `i18n.key-extra` (`todo`): a key is absent from the default dictionary.
- `i18n.dictionary-orphan` (`todo`): a dictionary exists for a language outside `aarshi.toml`.

### pages

- `pages.route-missing` (`error`): a view without a route.
- `pages.route-orphan` (`error`): a route without a view.
- `pages.title-key-missing` (`error`): the `"<page>.title"` key is missing from a dictionary.
- `pages.sitemap-entry-missing` (`error`): a page is missing from `app/sitemap.ts`.
- `pages.sitemap-entry-orphan` (`error`): a sitemap entry without a page.
- `pages.view-import-missing` (`error`): the view import is missing or incorrect in the route.
- `pages.import-broken` (`error`): a broken import in a view or a route.

### sections

- `sections.file-missing` (`error`): a dangling import (view → missing `sections/<section>.tsx`).
- `sections.title-key-missing` (`error`): the `"<page>.<section>.title"` key is missing from a dictionary.
- `sections.marker-missing` (`error`): the `// aarshi:imports:end` or `{/* aarshi:sections:end */}` marker is missing from the view.
- `sections.orphan` (`todo`): a section never imported.
- `sections.import-broken` (`error`): a broken import in a section.

### components

- `components.icon-unconverted` (`todo`): a `.svg` without a generated component.
- `components.import-broken` (`error`): a broken import in `components/`.

### dependencies

- `dependencies.package-missing` (`error`): an imported package not declared in `package.json` (`@sglara/cn`, `next`, `react`, `@base-ui/react`, `lucide-react`, ...).
- `dependencies.lockfile-mismatch` (`error`): a lockfile inconsistent with `package_manager`.
- `dependencies.modules-missing` (`todo`): `node_modules/` is missing.
