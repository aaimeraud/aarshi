---
labels: [wayfinder:grilling]
parent: 0014-map-implementation-status.md
blocked-by: [0015-grilling-perimetre-implementation.md]
status: open
---

# Modèle interne du moteur de `status`

## Question

Quelle structure de code pour `status` ? Notamment :

- **Le modèle des constats.** `Finding` (`rule`, `severity`, `summary`, `detail`, `location`), `Dimension` (six identifiants dans l'ordre canonique), `Severity` (`error`/`todo`), `Location` (`file`, `locale`, `key` ; `line` réservé, absent en v1). Où vivent les types et les 36 identifiants de règle : un module `src/status.rs`, un dossier `src/status/` éclaté par dimension, ou autre ?
- **L'organisation des contrôles.** Une fonction par dimension, par règle, ou une passe par objet ? Comment le moteur les itère, collecte tous les constats (la règle transverse interdit tout arrêt anticipé), puis trie (par `location.file` puis identifiant de règle) et compte.
- **Les primitives de lecture partagées** (texte, sans parseur TS) : extraction d'imports, clés de dictionnaire (aujourd'hui `dictionary::entries`, à étendre pour les doublons et les clés en trop), scan récursif des routes sous `app/[locale]/`, entrées de sitemap, marqueurs. Que reprend-on de `src/add.rs`, `src/dictionary.rs`, `src/check.rs`, et qu'introduit-on ?
- **Le seam d'extension.** Comment laisser ouverte l'extensibilité (contrôles déclarés par le dev, listée dans « Not yet specified ») sans l'implémenter ni complexifier le moteur v1 ?
- **Le point d'entrée.** Signature de `status::run(...)` : que reçoit-il (dossier, `Config`, options), que renvoie-t-il (modèle de rapport) et où sont calculés filtre, compteurs et code de sortie.

La **forme de sortie** (rendu humain, `--json`, wording) et la **plomberie des codes** sont tranchées par les tickets « Rendu humain, `--json` et wording des messages » et « Codes de sortie et absorption de `check` ».

## Resolution
