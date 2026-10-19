<p align="center">
  <img src="assets/aarshi-icon.png" alt="aarshi" width="96">
</p>

# aarshi

[English](README.md) | **Français**

[![CI](https://github.com/aaimeraud/aarshi/actions/workflows/ci.yml/badge.svg)](https://github.com/aaimeraud/aarshi/actions/workflows/ci.yml)

aarshi est un CLI en Rust qui génère un site [Next.js](https://nextjs.org) multilingue (App Router), puis vous aide à le garder en ordre. Il écrit une fois l'arborescence de routes, les dictionnaires typés, le plan du site et le sélecteur de langue, et ajoute ensuite des pages, des sections, des langues et des icônes sans copier de fichiers à la main.

## Pourquoi aarshi

Un site Next.js multilingue, c'est de la plomberie avant d'être du contenu : une route par langue et par page, un dictionnaire par langue, un plan du site qui doit rester synchronisé et un sélecteur dans l'en-tête. aarshi écrit cette plomberie une fois, et chaque ajout suit la même structure.

`aarshi status` vérifie le résultat. Une route, une clé de dictionnaire, une entrée de plan du site ou un import manquant est signalé avec un nom de règle stable et un code de sortie utilisable dans un script. Un projet cassé se voit avant la production.

## Prérequis

- **Rust 1.85 ou plus récent** pour compiler aarshi (édition 2024).
- **Node.js et un gestionnaire de paquets** (npm, pnpm, yarn ou bun) pour créer et lancer le site généré.

## Installer

aarshi n'est pas encore publié sur crates.io. Compilez-le depuis le dépôt :

```sh
git clone https://github.com/aaimeraud/aarshi.git
cd aarshi
cargo install --path .
```

Vérifiez l'installation :

```sh
aarshi --version
```

## Démarrage rapide

Créez un site en français et en anglais, avec le français par défaut :

```sh
aarshi setup portfolio \
  --languages fr,en \
  --default-language fr \
  --site-url https://example.com \
  --npm --yes
cd portfolio
```

Ajoutez une page À propos, une section équipe et un composant réutilisable :

```sh
aarshi add -p about
aarshi add -s team about
aarshi add -c member-card
```

Vérifiez le projet :

```sh
aarshi status
```

La page et la section ont pour l'instant des textes vides, donc `status` les liste en `todo`. Remplissez les valeurs dans `lib/i18n/dictionaries/fr.ts` et `en.ts`, relancez `aarshi status`, puis démarrez le site :

```sh
npm run dev
```

Ouvrez <http://localhost:3000/fr/about>, puis passez à `/en/about`.

## Commandes

| Commande | Ce qu'elle fait |
| --- | --- |
| `aarshi setup [<dossier>]` | Crée un projet Next.js. Demande les langues, l'URL du site, le gestionnaire de paquets, la bibliothèque d'interface et celle d'icônes, sauf si des options sont fournies. |
| `aarshi add` | Demande s'il faut ajouter une section, une page ou un composant, puis demande les noms. |
| `aarshi add -s <section> <page>` | Ajoute une section à une page. |
| `aarshi add -p <page>` | Ajoute une page : vue, route, entrée de plan du site et clés de dictionnaire. |
| `aarshi add -c <nom>` | Ajoute un composant de départ dans `components/custom/`. |
| `aarshi add-lang <code>` | Ajoute une langue et son dictionnaire. |
| `aarshi icons [--all]` | Convertit les SVG de `components/icons/` en composants React. |
| `aarshi status` | Signale les fichiers manquants ou incohérents. `check` est un alias ; `--json`, `--dimension` et `--errors-only` changent la sortie. |
| `aarshi remove` | Liste ce qu'aarshi a ajouté et supprime les pages, sections ou composants sélectionnés. `--all` et `--yes` évitent la sélection. |
| `aarshi restore` | Recopie ce que la dernière commande `remove` a retiré. |
| `aarshi docs` | Ouvre la documentation hors ligne dans le navigateur. |

Chaque commande demande les valeurs qui ne sont pas passées en option.

## Ce qu'il génère

- Une arborescence de routes commune sous `app/[locale]/`, avec `/` qui redirige vers la langue par défaut.
- `views/` pour les vues de page, `views/<page>/sections/` pour les sections, et des dictionnaires typés dans `lib/i18n/dictionaries/`.
- Un plan du site et un `robots.txt` générés à partir de la liste des langues.
- Un sélecteur de langue qui fonctionne avec ou sans shadcn.
- Tailwind, ESLint, TypeScript, un `AGENTS.md` et un `aarshi.toml` à la racine du projet.
- shadcn (basé sur Base UI) ou Base UI directement, et lucide-react ou aucune bibliothèque d'icônes.

Les conventions du projet généré, dont toutes les règles vérifiées par `aarshi status`, sont listées dans [CONVENTIONS.fr.md](CONVENTIONS.fr.md).

## Documentation

La documentation est incluse dans le binaire. Lancez :

```sh
aarshi docs
```

Elle s'ouvre localement, fonctionne hors ligne, et couvre `aarshi.toml`, chaque commande, un exemple complet, `status` et le dépannage, en anglais et en français.

## Contribuer

Les signalements, idées et corrections sont bienvenus. Ouvrez une issue avec le modèle de bug ou de demande de fonctionnalité, ou lisez [CONTRIBUTING.fr.md](CONTRIBUTING.fr.md) pour compiler, tester et proposer une pull request.
