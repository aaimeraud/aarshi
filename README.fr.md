# aarshi

Un CLI en Rust qui génère des sites Next.js multilingues (App Router) avec `views/`, `sections/`, une route par langue et des dictionnaires typés. Le projet généré est en TypeScript.

Statut : spécification. Le CLI n'est pas encore implémenté. Voir [`issues/0001-map.md`](issues/0001-map.md) pour les décisions et les points ouverts.

Version anglaise : [`README.md`](README.md). Conventions : [`CONVENTIONS.fr.md`](CONVENTIONS.fr.md).

## Commandes (v1)

- `aarshi setup [<dossier>]` : crée un projet Next.js. Sans argument, demande le nom du dossier. `aarshi setup .` utilise le dossier courant.
- `aarshi add -s <section> <page>` : ajoute une section à une page. Si la page n'existe pas, signale l'erreur et propose de la créer (o/n).
- `aarshi add -p <page>` : ajoute une page.

Gestionnaire de paquets : `--npm` (défaut), `--pnpm`, `--yarn`, `--bun`.

Les composants externes (shadcn) sont saisis librement et passés à `shadcn add`.

## Projet généré

- Créé avec `create-next-app`, avec des flags explicites : TypeScript, Tailwind, App Router, sans `src/`, ESLint, `AGENTS.md`. L'agent feedback et les Cache Components sont désactivés par défaut et peuvent être activés par un flag.
- Télémétrie désactivée pour le projet : `NEXT_TELEMETRY_DISABLED=1` est défini pendant la création, puis écrit dans le `.env` du projet. La configuration globale de Next.js n'est pas modifiée.
- Langues : fr, en, es, ko par défaut, extensibles par un fichier de config. Le sens d'écriture est déduit de chaque langue (ar, he, fa sont RTL).
- Le sitemap est mis à jour via des marqueurs de commentaire dans `app/sitemap.ts`.

## Installation

Prévu : `cargo install` ou Homebrew, sous forme de binaire unique.

## Non spécifié

- Mise à jour du sitemap au-delà des marqueurs de commentaire.
- Options de `add -p` au-delà de la création de page, de section et d'entrée de sitemap.
- Format et emplacement du fichier de config des langues.
- Contenu exact des templates et des fichiers générés par `setup`.
