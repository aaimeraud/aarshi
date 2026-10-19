<p align="center">
  <img src="assets/aarshi-icon.png" alt="aarshi" width="96">
</p>

# aarshi

**English** | [Français](README.fr.md)

[![CI](https://github.com/aaimeraud/aarshi/actions/workflows/ci.yml/badge.svg)](https://github.com/aaimeraud/aarshi/actions/workflows/ci.yml)

aarshi is a Rust CLI that scaffolds a multilingual [Next.js](https://nextjs.org) App Router site, then helps you keep it in order. It writes the route tree, the typed dictionaries, the sitemap and the language switcher once, and adds pages, sections, languages and icons without copying files by hand.

## Why aarshi

A multilingual Next.js site is plumbing before it is content: one route per language and per page, a dictionary per language, a sitemap that must stay in sync, and a switcher in the header. aarshi writes that plumbing once, and every later addition follows the same structure.

`aarshi status` checks the result. Missing routes, dictionary keys, sitemap entries or imports are reported with a stable rule name, and an exit code you can use in a script. A broken project is visible before it reaches production.

## Requirements

- **Rust 1.85 or newer** to build aarshi (edition 2024).
- **Node.js and a package manager** (npm, pnpm, yarn or bun) to create and run the generated site.

## Install

aarshi is not published on crates.io yet. Build it from the repository:

```sh
git clone https://github.com/aaimeraud/aarshi.git
cd aarshi
cargo install --path .
```

Check the installation:

```sh
aarshi --version
```

## Quick start

Create a site in French and English, with French as the default language:

```sh
aarshi setup portfolio \
  --languages fr,en \
  --default-language fr \
  --site-url https://example.com \
  --npm --yes
cd portfolio
```

Add an About page, a team section and a reusable component:

```sh
aarshi add -p about
aarshi add -s team about
aarshi add -c member-card
```

Check the project:

```sh
aarshi status
```

The page and the section have empty text entries at this point, so `status` lists them as `todo`. Fill the values in `lib/i18n/dictionaries/fr.ts` and `en.ts`, run `aarshi status` again, then start the site:

```sh
npm run dev
```

Open <http://localhost:3000/fr/about>, then switch to `/en/about`.

## Commands

| Command | What it does |
| --- | --- |
| `aarshi setup [<dir>]` | Creates a Next.js project. Asks for the languages, site URL, package manager, UI library and icon library unless flags are given. |
| `aarshi add` | Asks whether to add a section, a page or a component, then asks for the names. |
| `aarshi add -s <section> <page>` | Adds a section to a page. |
| `aarshi add -p <page>` | Adds a page: view, route, sitemap entry and dictionary keys. |
| `aarshi add -c <name>` | Adds a starter component in `components/custom/`. |
| `aarshi add-lang <code>` | Adds a language and its dictionary. |
| `aarshi icons [--all]` | Converts the SVGs of `components/icons/` to React components. |
| `aarshi status` | Reports missing or inconsistent files. `check` is an alias; `--json`, `--dimension` and `--errors-only` change the output. |
| `aarshi remove` | Lists what aarshi added and removes the selected pages, sections or components. `--all` and `--yes` skip the picking. |
| `aarshi restore` | Puts back what the last `remove` took out. |
| `aarshi docs` | Opens the offline documentation in the browser. |

Every command asks for the values it needs when they are not passed as flags.

## What it generates

- One shared route tree under `app/[locale]/`, with `/` redirecting to the default language.
- `views/` for page views, `views/<page>/sections/` for sections, and typed dictionaries in `lib/i18n/dictionaries/`.
- A sitemap and a `robots.txt` generated from the language list.
- A language switcher that works with or without shadcn.
- Tailwind, ESLint, TypeScript, an `AGENTS.md` and an `aarshi.toml` at the project root.
- shadcn (Base UI based) or Base UI directly, and lucide-react or no icon library.

The conventions of the generated project, including every rule checked by `aarshi status`, are listed in [CONVENTIONS.md](CONVENTIONS.md).

## Documentation

The documentation is embedded in the binary. Run:

```sh
aarshi docs
```

It opens locally, works offline, and covers `aarshi.toml`, every command, a complete example workflow, `status` and troubleshooting, in English and French.

## Contributing

Reports, ideas and fixes are welcome. Open an issue with the bug report or feature request template, or read [CONTRIBUTING.md](CONTRIBUTING.md) for the build, test and pull request workflow.
