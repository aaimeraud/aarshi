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

## Not yet specified

- Mécanisme de mise à jour du sitemap au-delà des marqueurs de commentaire.
- Options de `add -p` au-delà de la création de page, de section et de l'entrée sitemap.
- Clés exactes des fichiers `config.toml` et `aarshi.toml`.
- Langue de l'interface du CLI.
- Contenu exact des templates (section, vue, route, dictionnaire typé) et des premiers fichiers générés par `setup`.

## Out of scope

- Commandes au-delà du périmètre v1.
