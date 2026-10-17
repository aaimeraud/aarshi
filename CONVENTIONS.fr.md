# Conventions

Conventions pour le dépôt `aarshi` et pour les projets qu'il génère. Version anglaise : [`CONVENTIONS.md`](CONVENTIONS.md).

## Dépôt

- Langue : documentation en anglais (`README.md`, `CONVENTIONS.md`) avec une version française (`*.fr.md`). Code et commentaires de code en anglais.
- Les commentaires sont courts et uniquement là où le comportement n'est pas évident. Les fonctions Rust publiques ont un commentaire `///`.
- Stack : Rust, `clap` pour les commandes, `inquire` pour les questions interactives, un binaire unique.
- Le travail de spécification est suivi en issues numérotées dans `issues/` (`NNNN-slug.md`), avec un frontmatter (`labels`, `status`, `parent`, `blocked-by`). La carte est `issues/0001-map.md`.
- Les messages de commit suivent `type(scope): summary`.

## Projets générés

- TypeScript, Next.js App Router, Tailwind, ESLint, sans dossier `src/`.
- `AGENTS.md` est généré à la racine du projet.
- La télémétrie est désactivée par projet (`.env`), pas globalement.
- Structure : `views/` pour les vues de page, `sections/` pour les sections, une route par langue, dictionnaires typés par langue.
- Le sens d'écriture (`ltr` ou `rtl`) vient de la langue, pas de la page.

## Non spécifié

- Nommage exact des templates et des fichiers générés (voir la carte).

