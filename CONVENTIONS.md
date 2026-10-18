# Conventions

Conventions for the `aarshi` repository and for the projects it generates. French version: [`CONVENTIONS.fr.md`](CONVENTIONS.fr.md).

## Repository

- Language: documentation in English (`README.md`, `CONVENTIONS.md`) with a French counterpart (`*.fr.md`). Code and code comments in English.
- Comments are short and only where the behavior is not obvious. Public Rust functions have a `///` doc comment.
- Stack: Rust, `clap` for commands, `inquire` for interactive questions, one binary.
- Specification work is tracked as numbered issues in `issues/` (`NNNN-slug.md`), with frontmatter (`labels`, `status`, `parent`, `blocked-by`). The map is `issues/0001-map.md`.
- Commit messages follow `type(scope): summary`.

## Generated projects

- TypeScript, Next.js App Router, Tailwind, ESLint, no `src/` folder.
- `AGENTS.md` is generated at the project root.
- Telemetry is disabled per project (`.env`), not globally.
- Structure: `views/` for page views, `sections/` for sections, one shared route tree under `app/[locale]/`, typed dictionaries per language.
- Text direction (`ltr` or `rtl`) comes from the language, not from the page.
- Components: shadcn (styled, Base UI based) by default, or Base UI directly. Class names go through `cn` from `@sglara/cn`. `components/custom/` holds project components, `components/icons/` the SVGs converted by `aarshi icons`.
- Generated TypeScript uses arrow functions and named exports for sections, icons and custom components.

## Not yet specified

- Exact naming of templates and generated files (see the map).

