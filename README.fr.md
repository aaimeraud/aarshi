# aarshi

Un CLI en Rust qui génère des sites Next.js multilingues (App Router) avec `views/`, `sections/`, une arborescence de routes commune par préfixe de langue et des dictionnaires typés. Le projet généré est en TypeScript.

Statut : le CLI v1 est en revue (PR #4), et `aarshi status` est désormais implémenté. Voir [`issues/0001-map.md`](issues/0001-map.md) pour les décisions et les points ouverts.

Version anglaise : [`README.md`](README.md). Conventions : [`CONVENTIONS.fr.md`](CONVENTIONS.fr.md).

## Commandes

Chaque commande demande les valeurs qui ne sont pas passées en argument. Passer `--yes` à `setup` accepte les valeurs par défaut.

- `aarshi setup [<dossier>]` : crée un projet Next.js. `aarshi setup .` utilise le dossier courant. Demande les langues (cases à cocher parmi fr, en, es, ja, ko, de, puis la langue par défaut), l'URL du site, le gestionnaire de paquets (`--npm`, `--pnpm`, `--yarn`, `--bun`), les composants d'interface (shadcn ou Base UI), la bibliothèque d'icônes (lucide-react ou aucune) et les options (agent feedback, Cache Components, sous forme de cases à cocher). `--languages fr,en,ja`, `--default-language ja`, `--ui shadcn|base-ui` et `--icons lucide|none` évitent ces questions.
- `aarshi add` : demande si l'on ajoute une section, une page ou un composant, puis demande les noms.
- `aarshi add -s <section> <page>` : ajoute une section à une page. La page est choisie dans la liste des pages de `views/` ; la première entrée en crée une nouvelle. Si une page est passée en argument et n'existe pas, la commande signale l'erreur et propose de la créer (o/n).
- `aarshi add -p <page>` : ajoute une page.
- `aarshi add -c <nom>` : ajoute un composant de départ dans `components/custom/`.
- `aarshi add-lang <code>` : ajoute une langue (par ex. `de`) à un projet existant. Crée son dictionnaire avec des valeurs vides et met à jour la liste des langues.
- `aarshi docs` : écrit la documentation (HTML, incluse dans le binaire) dans un dossier temporaire, vidé à chaque appel, et l'ouvre dans le navigateur. La page est en français quand la langue du système commence par `fr`, en anglais sinon.
- `aarshi icons` : liste les fichiers `.svg` de `components/icons/` sous forme de cases à cocher et convertit ceux qui sont sélectionnés en composants React (`--all` convertit tout sans rien demander). `class` devient `className`, les attributs passent en camelCase, les styles en ligne deviennent des objets, et les valeurs simples passent en classes Tailwind (`width`/`height` vers `size-*`, `fill="currentColor"` vers `fill-current`, `fill="none"` vers `fill-none`, `stroke="currentColor"` vers `stroke-current`). Un composant existant n'est jamais écrasé.

### aarshi status

Signale ce qui manque ou ce qui ne va pas dans le projet généré. Un diagnostic en lecture seule : il ne modifie rien. `check` est un alias visible affiché dans l'aide, et `aarshi check` produit exactement le même rapport que `aarshi status`.

Le rapport affiche une ligne par dimension, dans l'ordre canonique (`config`, `i18n`, `pages`, `sections`, `components`, `dependencies`). Une dimension sans rien à signaler reçoit un `✓` ; les constats sont indentés sous leur dimension et préfixés `error:` (rouge) ou `todo:` (jaune). La dernière ligne est le verdict sur le projet entier : `✓ project is conformant`, `✓ project is conformant (N to do)` quand il ne reste que des constats `todo:`, ou `error: N errors, M to do` quand au moins une règle est cassée.

- Codes de sortie : `0` quand aucune `error` n'a été trouvée (des problèmes de gravité `todo` seuls laissent `0`), `1` quand au moins une `error` a été trouvée, `2` quand le projet ne peut pas être analysé (`aarshi.toml` absent, illisible ou invalide) : le message va sur stderr et rien n'est écrit sur stdout.
- `--dimension <nom>` : restreint le rapport aux dimensions données. Le flag est répétable et accepte les virgules (`--dimension config,i18n`) ; les valeurs sont les six identifiants canoniques : `config`, `i18n`, `pages`, `sections`, `components`, `dependencies`. Seul l'affichage est filtré : le code de sortie et le résumé JSON restent calculés sur le projet entier, et une ligne `info: N findings hidden by --dimension` précise combien de constats le filtre masque.
- `--errors-only` : masque les constats `todo:`. Le verdict et le code de sortie restent inchangés.
- `--json` : écrit un document JSON unique sur stdout : `format_version` (actuellement `1`), un `summary` global (`errors`, `todos`, `conformant`) calculé sur le projet entier, et les `dimensions` sélectionnées — six sauf filtre, chacune avec ses compteurs et ses `findings`. Un constat porte `rule` (un identifiant stable de forme `<dimension>.<sujet>-<problème>`), `severity` (`error` ou `todo`), `summary` (la ligne montrée à l'humain), `detail` et `location` optionnels, cette dernière ne portant que les clés pertinentes (`file`, `locale`, `key`). En cas de sortie code `2`, stdout reste vide.

Le catalogue des règles vérifiées par `status` est listé par dimension dans [`CONVENTIONS.fr.md`](CONVENTIONS.fr.md).

## Projet généré

- Créé avec `create-next-app`, avec des flags explicites : TypeScript, Tailwind, App Router, sans `src/`, ESLint, `AGENTS.md`. L'agent feedback et les Cache Components sont désactivés par défaut et peuvent être activés par un flag.
- Télémétrie désactivée pour le projet : `NEXT_TELEMETRY_DISABLED=1` est défini pendant la création, puis écrit dans le `.env` du projet. La configuration globale de Next.js n'est pas modifiée.
- Langues : choisies pendant `setup` parmi fr, en, es, ja, ko et de (fr et en sont précochées, puis `setup` demande laquelle est la langue par défaut). La première langue de la liste est la langue par défaut. La liste se règle dans la config utilisateur (`~/.config/aarshi/config.toml`) et peut être remplacée par projet grâce à `aarshi.toml` à la racine du projet, qui remplace la liste. Chaque langue a un préfixe (`/fr`, `/en`), et `/` redirige vers la langue par défaut. Le sens d'écriture est déduit de chaque langue (ar, he, fa sont RTL).
- Un seul layout racine sous `app/[locale]/` définit `lang` et `dir`. Le sélecteur de langue dans l'en-tête est un `Menu` de Base UI (une entrée par langue, rendue en lien), donc il change de langue par navigation côté client et fonctionne avec ou sans shadcn.
- Composants : `shadcn` (stylisé, basé sur Base UI, par défaut) ou Base UI directement. `cn` vient de `@sglara/cn`. `components/custom/` contient les composants propres au projet et `components/icons/` les SVG convertis.
- Le sitemap est mis à jour via des marqueurs de commentaire dans `app/sitemap.ts`.

## Installation

Prévu : `cargo install` ou Homebrew, sous forme de binaire unique.

## Non spécifié

- Mise à jour du sitemap au-delà des marqueurs de commentaire.
- Options de `add -p` au-delà de la création de page, de section et d'entrée de sitemap.
- Texte littéral des templates générés (la structure est décidée).
