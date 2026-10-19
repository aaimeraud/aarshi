# aarshi

A Rust CLI that scaffolds multilingual Next.js sites (App Router) with `views/`, `sections/`, one shared route tree per language prefix, and typed dictionaries. The generated project is TypeScript.

Status: the v1 CLI is in review (PR #4), and `aarshi status` is now implemented. See [`issues/0001-map.md`](issues/0001-map.md) for the decisions and open points.

French version: [`README.fr.md`](README.fr.md). Conventions: [`CONVENTIONS.md`](CONVENTIONS.md).

## Commands

Every command asks for the values it needs when they are not given as arguments. Pass `--yes` to `setup` to accept the defaults instead.

- `aarshi setup [<dir>]`: creates a Next.js project. `aarshi setup .` uses the current folder. Asks for the languages (checkboxes among fr, en, es, ja, ko, de, then the default one), the site URL, the package manager (`--npm`, `--pnpm`, `--yarn`, `--bun`), the UI components (shadcn or Base UI), the icon library (lucide-react or none) and the options (agent feedback, Cache Components, as checkboxes). `--languages fr,en,ja`, `--default-language ja`, `--ui shadcn|base-ui` and `--icons lucide|none` skip those questions.
- `aarshi add`: asks whether to add a section, a page or a custom component, then asks for the names.
- `aarshi add -s <section> <page>`: adds a section to a page. The page is picked from the list of pages in `views/`; the first entry creates a new one. If a page is given as an argument and does not exist, it reports the error and asks whether to create it (y/n).
- `aarshi add -p <page>`: adds a page.
- `aarshi add -c <name>`: adds a starter component in `components/custom/`.
- `aarshi add-lang <code>`: adds a language (e.g. `de`) to an existing project. Creates its dictionary with empty values and updates the language list.
- `aarshi docs`: writes the documentation (HTML, embedded in the binary) to a temporary folder, emptied on every call, and opens it in the browser. The page is in French when the system language starts with `fr`, in English otherwise.
- `aarshi icons`: lists the `.svg` files of `components/icons/` as checkboxes and converts the selected ones to React components (`--all` converts every file without asking). `class` becomes `className`, attributes become camelCase, inline styles become objects, and simple values become Tailwind classes (`width`/`height` to `size-*`, `fill="currentColor"` to `fill-current`, `fill="none"` to `fill-none`, `stroke="currentColor"` to `stroke-current`). An existing component is never overwritten.
- `aarshi remove`: lists what aarshi added (pages, sections and custom components) as checkboxes and removes the selected ones. `--all` selects everything, and `--yes` skips the confirmation. The home page is never offered. Before touching anything, it copies every file it will change into `.aarshi/removed/` and adds that folder to the project's `.gitignore`.
- `aarshi restore`: puts back the files of the last `remove` and forgets it. `--yes` skips the confirmation. A section of a removed page comes back with the page.

### aarshi status

Reports what is missing or wrong in the generated project. A read-only diagnostic: it changes nothing. `check` is a visible alias shown in the help, and `aarshi check` produces exactly the same report as `aarshi status`.

The report prints one line per dimension, in canonical order (`config`, `i18n`, `pages`, `sections`, `components`, `dependencies`). A dimension with nothing to report gets a `✓`; the findings are indented under their dimension and prefixed `error:` (red) or `todo:` (yellow). The last line is the verdict on the whole project: `✓ project is conformant`, `✓ project is conformant (N to do)` when only `todo:` findings remain, or `error: N errors, M to do` when at least one rule is broken.

- Exit codes: `0` when no `error` was found (issues of severity `todo` alone still exit `0`), `1` when at least one `error` was found, `2` when the project cannot be analyzed (no, unreadable or invalid `aarshi.toml`): the message goes to stderr and nothing is written to stdout.
- `--dimension <name>`: restricts the report to the given dimensions. The flag is repeatable and accepts commas (`--dimension config,i18n`); the values are the six canonical ids: `config`, `i18n`, `pages`, `sections`, `components`, `dependencies`. The display only is filtered: the exit code and the JSON summary are still computed on the whole project, and a line `info: N findings hidden by --dimension` states how many findings the filter hides.
- `--errors-only`: hides the `todo:` findings. The verdict and the exit code are unchanged.
- `--json`: writes a single JSON document on stdout: `format_version` (currently `1`), a global `summary` (`errors`, `todos`, `conformant`) computed on the whole project, and the selected `dimensions` — six unless filtered, each with its counters and its `findings`. A finding carries `rule` (a stable `<dimension>.<subject>-<problem>` identifier), `severity` (`error` or `todo`), `summary` (the line shown to the human), optional `detail` and optional `location` holding only the relevant keys (`file`, `locale`, `key`). On exit code `2`, stdout stays empty.

The catalogue of rules verified by `status` is listed by dimension in [`CONVENTIONS.md`](CONVENTIONS.md).

## Generated project

- Created with `create-next-app`, using explicit flags: TypeScript, Tailwind, App Router, no `src/`, ESLint, `AGENTS.md`. Agent feedback and Cache Components are off by default and can be enabled by a flag.
- Telemetry is disabled for the project: `NEXT_TELEMETRY_DISABLED=1` is set during creation and written to the project's `.env`. The global Next.js configuration is not changed.
- Languages: chosen during `setup` among fr, en, es, ja, ko and de (fr and en are preselected, and `setup` then asks which one is the default). The first language of the list is the default one. The list is set in the user config (`~/.config/aarshi/config.toml`) and can be overridden per project by `aarshi.toml` at the project root, which replaces the list. Every language has a prefix (`/fr`, `/en`), and `/` redirects to the default language. Text direction is derived per language (ar, he, fa are RTL).
- One root layout under `app/[locale]/` sets `lang` and `dir`. The language switcher in the header is a Base UI `Menu` (one item per language, rendered as a link), so it changes language with client-side navigation and works with or without shadcn.
- Components: `shadcn` (styled, Base UI based, the default) or Base UI directly. `cn` comes from `@sglara/cn`. `components/custom/` holds the project's own components and `components/icons/` the converted SVGs.
- The sitemap is updated through comment markers in `app/sitemap.ts`.

## Install

Planned: `cargo install` or Homebrew, as a single binary.

## Not yet specified

- How the sitemap is updated beyond comment markers.
- Options of `add -p` beyond page, section and sitemap entry creation.
- Exact literal content of the generated templates (the structure is decided).
