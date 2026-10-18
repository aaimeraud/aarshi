---
labels: [wayfinder:map]
status: open
parent: 0001-map.md
---

# Carte : implémentation du CLI aarshi v1

## Destination

Un binaire Rust `aarshi` qui réalise le périmètre v1 de la spec : `setup`, `add -s`, `add -p`, avec des templates embarqués, conforme à `README.md` et `CONVENTIONS.md`.

## Notes

- La spec est décrite dans `README.md` / `README.fr.md` et `CONVENTIONS.md` / `CONVENTIONS.fr.md`. Les décisions sont dans `issues/0001-map.md`.
- Stack : Rust, `clap`, `inquire`, binaire unique.
- Le projet généré est en TypeScript (Next.js App Router, Tailwind, ESLint, sans `src/`, `AGENTS.md`).

## Decisions so far

- Hérite de toutes les décisions de `issues/0001-map.md`.
- Config des langues : format TOML.
- Config utilisateur `~/.config/aarshi/config.toml` : langues par défaut des nouveaux projets. Les langues sont celles du site généré, pas celles de l'interface du CLI.
- Config projet `aarshi.toml` à la racine du projet généré : sa liste de langues remplace entièrement celle de l'utilisateur.
- Clés de config : `languages` (liste) et `package_manager` (optionnelle, un flag CLI l'emporte). `aarshi.toml` accepte les mêmes clés.
- Langues proposées par `setup` : fr, en, es, ja, ko, de, sous forme de cases à cocher (fr et en précochées). Seuls les dictionnaires des langues cochées sont créés.
- Après les langues, `setup` demande laquelle est la langue par défaut. Elle est placée en tête de la liste et sert de référence pour typer les dictionnaires.
- Chaque langue est préfixée (`/fr`, `/en`), y compris la langue par défaut ; `/` redirige vers elle. Une seule arborescence de routes, sous `app/[locale]/`, avec un seul layout racine qui définit `lang` et `dir`.
- Composants : `shadcn` (par défaut, stylisé, basé sur Base UI) ou Base UI directement. `cn` vient de `@sglara/cn`, installé dans tous les cas. Icônes : `lucide-react` par défaut, ou rien.
- `aarshi icons` convertit les `.svg` de `components/icons/` en composants React, avec cases à cocher (`--all` pour tout convertir) : attributs en camelCase, styles en ligne en objets, et classes Tailwind pour les cas simples.
- `aarshi add -c <nom>` crée un composant dans `components/custom/`.
- `aarshi add -s` choisit la page cible dans une liste tirée de `views/`, dont la première entrée crée une nouvelle page.
- Langue de l'interface du CLI : anglais.
- `setup` demande `SITE_URL` (défaut `http://localhost:3000`) et génère `robots.ts` en plus du sitemap. Pas de JSON-LD ni de dossier `llms/` en v1.

## Not yet specified

- Mécanisme de mise à jour du sitemap au-delà des marqueurs de commentaire.
- Options de `add -p` au-delà de la création de page, de section et de l'entrée sitemap.
- Texte littéral des templates générés (la structure est décidée, pas le contenu exact).

## Out of scope

- Commandes au-delà du périmètre v1.
