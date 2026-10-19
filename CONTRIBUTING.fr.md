# Contribuer à aarshi

Merci de prendre le temps de signaler un problème ou de proposer une modification. Ce guide concerne le dépôt aarshi. Les conventions des sites qu'il génère sont dans [CONVENTIONS.fr.md](CONVENTIONS.fr.md). Version anglaise : [CONTRIBUTING.md](CONTRIBUTING.md).

## Prérequis

- **Rust 1.85 ou plus récent** (édition 2024).
- **Node.js et un gestionnaire de paquets** pour essayer `aarshi setup` de bout en bout.

## Compiler et tester

```sh
cargo build
cargo test
```

Avant d'ouvrir une pull request, lancez ce que lance la CI :

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all --locked
```

## Signaler un bug

Ouvrez une issue avec le modèle **bug report**. Un bon rapport donne :

1. la commande exacte utilisée, options comprises ;
2. la sortie de `aarshi --version` et votre système d'exploitation ;
3. ce que vous attendiez et ce qui s'est passé, avec la sortie complète ;
4. pour un problème dans un projet, la sortie de `aarshi status --json` lancée dans ce projet.

Retirez avant tout ce qui est privé, comme une URL de site que vous ne voulez pas partager.

## Proposer une fonctionnalité

Décrivez d'abord le problème, puis la solution : que vouliez-vous faire, et combien cela coûte de le faire à la main. Une commande ou une option proposée est bienvenue, mais c'est le problème qui décide si la modification est retenue.

## Proposer une modification

1. Partez de la branche `dev` la plus récente, sur une branche au nom clair, par exemple `fix/invite-section`.
2. Faites la modification et ajoutez ou mettez à jour un test pour elle.
3. Lancez les vérifications ci-dessus.
4. Ouvrez une pull request qui explique ce qui change et pourquoi, et lie l'issue à laquelle elle répond.

Gardez une pull request ciblée. Une modification qui mélange un correctif, un refactor et une fonctionnalité est difficile à relire.

## Ce qu'une modification doit contenir

- Un test pour le comportement, dans `tests/cli.rs` pour le comportement de la CLI.
- La documentation en anglais **et** en français : README, CONTRIBUTING, CONVENTIONS et `src/docs/` changent ensemble.
- Pour une nouvelle règle de `status` : la règle listée dans [CONVENTIONS.fr.md](CONVENTIONS.fr.md) et [CONVENTIONS.md](CONVENTIONS.md), plus son test.

## Style

- Le code et les commentaires de code sont en anglais. La CLI affiche en anglais. La documentation est écrite en anglais et en français.
- Les commentaires sont courts et uniquement là où le comportement n'est pas évident. Les fonctions Rust publiques ont un commentaire de documentation `///`.
- Les messages de commit suivent `type(scope): summary`, par exemple `fix(status): exit 2 when aarshi.toml cannot be analyzed`.
- Le travail de spécification et les décisions de conception sont notés sous forme de tickets numérotés dans [`issues/`](issues/) (`NNNN-slug.md`). Liez le ticket quand une modification en découle.
