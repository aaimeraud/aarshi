# Contributing to aarshi

Thanks for taking the time to report a problem or propose a change. This guide covers the aarshi repository. The conventions of the sites it generates are in [CONVENTIONS.md](CONVENTIONS.md). French version: [CONTRIBUTING.fr.md](CONTRIBUTING.fr.md).

## Requirements

- **Rust 1.85 or newer** (edition 2024).
- **Node.js and a package manager** to try `aarshi setup` from end to end.

## Build and test

```sh
cargo build
cargo test
```

Before opening a pull request, run what CI runs:

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all --locked
```

## Report a bug

Open an issue with the **bug report** template. A good report gives:

1. the exact command you ran, flags included;
2. the output of `aarshi --version` and your operating system;
3. what you expected and what happened, with the full output;
4. for a problem inside a project, the output of `aarshi status --json` run in that project.

Remove anything private before pasting, such as a site URL you do not want to share.

## Propose a feature

Describe the problem before the solution: what you were trying to do, and what it costs to do it by hand. A proposed command or flag is welcome, but the problem decides whether the change is accepted.

## Propose a change

1. Start from the latest `dev` branch, on a branch with a clear name, for example `fix/add-section-prompt`.
2. Make the change and add or update a test for it.
3. Run the checks above.
4. Open a pull request that says what changed and why, and links the issue it answers.

Keep a pull request focused. A change that mixes a fix, a refactor and a new feature is hard to review.

## What a change must include

- A test for the behaviour, in `tests/cli.rs` for CLI behaviour.
- The documentation in English **and** French: README, CONTRIBUTING, CONVENTIONS and `src/docs/` change together.
- For a new `status` rule: the rule listed in [CONVENTIONS.md](CONVENTIONS.md) and [CONVENTIONS.fr.md](CONVENTIONS.fr.md), plus its test.

## Style

- Code and code comments are in English. The CLI prints in English. The documentation is written in English and French.
- Comments are short and only where the behaviour is not obvious. Public Rust functions have a `///` doc comment.
- Commit messages follow `type(scope): summary`, for example `fix(status): exit 2 when aarshi.toml cannot be analyzed`.
- Specification work and design decisions are kept as numbered tickets in [`issues/`](issues/) (`NNNN-slug.md`). Link the ticket when a change follows from one.
