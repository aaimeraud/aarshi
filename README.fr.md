# aarshi

Un CLI en Rust qui génère des sites Next.js multilingues (App Router) avec `views/`, `sections/`, une route par langue et des dictionnaires typés. Le projet généré est en TypeScript.

Statut : le CLI v1 est en revue (PR #4). Voir [`issues/0001-map.md`](issues/0001-map.md) pour les décisions et les points ouverts.

Version anglaise : [`README.md`](README.md). Conventions : [`CONVENTIONS.fr.md`](CONVENTIONS.fr.md).

## Commandes

Chaque commande demande les valeurs qui ne sont pas passées en argument. Passer `--yes` à `setup` accepte les valeurs par défaut.

- `aarshi setup [<dossier>]` : crée un projet Next.js. `aarshi setup .` utilise le dossier courant. Demande l'URL du site, le gestionnaire de paquets (`--npm`, `--pnpm`, `--yarn`, `--bun`) et les options (agent feedback, Cache Components, sous forme de cases à cocher).
- `aarshi add` : demande si l'on ajoute une section ou une page, puis demande les noms.
- `aarshi add -s <section> <page>` : ajoute une section à une page. Si la page n'existe pas, signale l'erreur et propose de la créer (o/n).
- `aarshi add -p <page>` : ajoute une page.
- `aarshi add-lang <code>` : ajoute une langue (par ex. `de`) à un projet existant. Crée son dictionnaire avec des valeurs vides et met à jour la liste des langues.
- `aarshi check` : liste les traductions vides, les clés manquantes et les routes manquantes. Ne modifie rien, et sort en erreur quand il trouve un problème.

Les composants externes (shadcn) ne sont pas encore pris en charge.

## Projet généré

- Créé avec `create-next-app`, avec des flags explicites : TypeScript, Tailwind, App Router, sans `src/`, ESLint, `AGENTS.md`. L'agent feedback et les Cache Components sont désactivés par défaut et peuvent être activés par un flag.
- Télémétrie désactivée pour le projet : `NEXT_TELEMETRY_DISABLED=1` est défini pendant la création, puis écrit dans le `.env` du projet. La configuration globale de Next.js n'est pas modifiée.
- Langues : fr, en, es, ko par défaut. La liste se règle dans la config utilisateur (`~/.config/aarshi/config.toml`) et peut être remplacée par projet grâce à `aarshi.toml` à la racine du projet, qui remplace la liste. Chaque langue a un préfixe (`/fr`, `/en`), et `/` redirige vers la première langue. Le sens d'écriture est déduit de chaque langue (ar, he, fa sont RTL).
- Un seul layout racine sous `app/[locale]/` définit `lang` et `dir`. Un sélecteur de langue dans l'en-tête change de langue par navigation côté client, donc la page ne recharge pas.
- Le sitemap est mis à jour via des marqueurs de commentaire dans `app/sitemap.ts`.

## Installation

Prévu : `cargo install` ou Homebrew, sous forme de binaire unique.

## Non spécifié

- Mise à jour du sitemap au-delà des marqueurs de commentaire.
- Options de `add -p` au-delà de la création de page, de section et d'entrée de sitemap.
- Texte littéral des templates générés (la structure est décidée).
