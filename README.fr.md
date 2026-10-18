# aarshi

Un CLI en Rust qui génère des sites Next.js multilingues (App Router) avec `views/`, `sections/`, une arborescence de routes commune par préfixe de langue et des dictionnaires typés. Le projet généré est en TypeScript.

Statut : le CLI v1 est en revue (PR #4). Voir [`issues/0001-map.md`](issues/0001-map.md) pour les décisions et les points ouverts.

Version anglaise : [`README.md`](README.md). Conventions : [`CONVENTIONS.fr.md`](CONVENTIONS.fr.md).

## Commandes

Chaque commande demande les valeurs qui ne sont pas passées en argument. Passer `--yes` à `setup` accepte les valeurs par défaut.

- `aarshi setup [<dossier>]` : crée un projet Next.js. `aarshi setup .` utilise le dossier courant. Demande les langues (cases à cocher parmi fr, en, es, ja, ko, de, puis la langue par défaut), l'URL du site, le gestionnaire de paquets (`--npm`, `--pnpm`, `--yarn`, `--bun`), les composants d'interface (shadcn ou Base UI), la bibliothèque d'icônes (lucide-react ou aucune) et les options (agent feedback, Cache Components, sous forme de cases à cocher). `--languages fr,en,ja`, `--default-language ja`, `--ui shadcn|base-ui` et `--icons lucide|none` évitent ces questions.
- `aarshi add` : demande si l'on ajoute une section, une page ou un composant, puis demande les noms.
- `aarshi add -s <section> <page>` : ajoute une section à une page. La page est choisie dans la liste des pages de `views/` ; la première entrée en crée une nouvelle. Si une page est passée en argument et n'existe pas, la commande signale l'erreur et propose de la créer (o/n).
- `aarshi add -p <page>` : ajoute une page.
- `aarshi add -c <nom>` : ajoute un composant de départ dans `components/custom/`.
- `aarshi add-lang <code>` : ajoute une langue (par ex. `de`) à un projet existant. Crée son dictionnaire avec des valeurs vides et met à jour la liste des langues.
- `aarshi icons` : liste les fichiers `.svg` de `components/icons/` sous forme de cases à cocher et convertit ceux qui sont sélectionnés en composants React (`--all` convertit tout sans rien demander). `class` devient `className`, les attributs passent en camelCase, les styles en ligne deviennent des objets, et les valeurs simples passent en classes Tailwind (`width`/`height` vers `size-*`, `fill="currentColor"` vers `fill-current`, `fill="none"` vers `fill-none`, `stroke="currentColor"` vers `stroke-current`). Un composant existant n'est jamais écrasé.
- `aarshi check` : liste les traductions vides, les clés manquantes et les routes manquantes. Ne modifie rien, et sort en erreur quand il trouve un problème.

## Projet généré

- Créé avec `create-next-app`, avec des flags explicites : TypeScript, Tailwind, App Router, sans `src/`, ESLint, `AGENTS.md`. L'agent feedback et les Cache Components sont désactivés par défaut et peuvent être activés par un flag.
- Télémétrie désactivée pour le projet : `NEXT_TELEMETRY_DISABLED=1` est défini pendant la création, puis écrit dans le `.env` du projet. La configuration globale de Next.js n'est pas modifiée.
- Langues : choisies pendant `setup` parmi fr, en, es, ja, ko et de (fr et en sont précochées, puis `setup` demande laquelle est la langue par défaut). La première langue de la liste est la langue par défaut. La liste se règle dans la config utilisateur (`~/.config/aarshi/config.toml`) et peut être remplacée par projet grâce à `aarshi.toml` à la racine du projet, qui remplace la liste. Chaque langue a un préfixe (`/fr`, `/en`), et `/` redirige vers la langue par défaut. Le sens d'écriture est déduit de chaque langue (ar, he, fa sont RTL).
- Un seul layout racine sous `app/[locale]/` définit `lang` et `dir`. Le sélecteur de langue dans l'en-tête est construit avec `ButtonGroup`, et ses liens reprennent les styles du bouton, donc il garde la sémantique d'un lien et change de langue par navigation côté client.
- Composants : `shadcn` (stylisé, basé sur Base UI, par défaut) ou Base UI directement. `cn` vient de `@sglara/cn`. `components/custom/` contient les composants propres au projet et `components/icons/` les SVG convertis.
- Le sitemap est mis à jour via des marqueurs de commentaire dans `app/sitemap.ts`.

## Installation

Prévu : `cargo install` ou Homebrew, sous forme de binaire unique.

## Non spécifié

- Mise à jour du sitemap au-delà des marqueurs de commentaire.
- Options de `add -p` au-delà de la création de page, de section et d'entrée de sitemap.
- Texte littéral des templates générés (la structure est décidée).
