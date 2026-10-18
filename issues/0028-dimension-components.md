---
labels: [wayfinder:implement]
parent: 0014-map-implementation-status.md
blocked-by: [0027-dimension-sections.md]
status: closed
---

# Dimension `components` : icônes non converties et resolved imports

## Tranche

Remplir la dimension `components` (catalogue [0007](0007-grilling-invariants-pages.md)) :

- `components.icon-unconverted` (`todo`) : `components/icons/<name>.svg` sans `<name>.tsx` (conversion pas encore faite). `<name>.tsx` sans `.svg` est **ignoré**.
- `components.import-broken` (`error`) : import cassé dans `components/`.
- **Résolution des imports des fichiers connus** : les imports des routes, vues, sections et `app/[locale]/layout.tsx` (y compris `language-switcher`) doivent résoudre vers un fichier existant — un composant référentiel manquant est un `error`, classé dans la dimension de l'objet fautif ; un composant de `components/custom/` **non utilisé n'est pas signalé**.

Cette tranche s'appuie sur les primitives d'imports et l'inventaire des fichiers connus posés par [0026](0026-dimension-pages.md)/[0027](0027-dimension-sections.md).

## Critères d'acceptance

- [x] Icône `.svg` sans `.tsx` → un `todo` ; `.tsx` sans `.svg` → rien ; les deux par tests noirs.
- [x] Un import d'un composant inexistant (vues/routes/sections/layout) produit un constat `error` dans la dimension de l'objet fautif, avec tests.
- [x] Un composant `custom` non importé ne génère **aucun** constat ; test noir.
- [x] La présence de `components/language-switcher.tsx` reste dans la dimension `config` ([0024](0024-dimension-config.md)) — pas de double comptage.

## Implémentation

Le collecteur vit dans `src/status/components.rs`, enregistré dans `COLLECTORS` entre `sections` et `dependencies`, sur les primitives de [0026](0026-dimension-pages.md) (`imports`, `resolve_module`, `is_bare`). Wording selon [0017](0017-grilling-rendu-json.md) : `` `components/icons/logo.svg` has no generated component `` (détail `expected components/icons/logo.tsx`) et le format commun `broken import \`<module>\` in \`<file>\`` (détail `no file matches \`<module>\``, identique aux dimensions `pages`/`sections`).

Convention appliquée pour la résolution des imports des fichiers connus : elle est classée par dimension de l'objet fautif et déjà couverte — une vue ou une route qui importe un composant manquant → `pages.import-broken` (0026), une section → `sections.import-broken` (0027), un fichier de base → la promenade de `config` (0024). Donc `components.import-broken` ne couvre ici que les imports **depuis** les fichiers de `components/custom/` et `components/icons/` ; les paquets nus restent à `dependencies` (0029) et les composants `custom` non utilisés ne génèrent rien. Exception figée du côté `config` : un fichier de base qui importe un composant manquant (`components/-*`) reste silencieux, conformément au test noir déjà figé de 0024 (« les `components` masters résolvent les imports de leurs fichiers ») ; de même, `components/language-switcher.tsx` est un fichier de base : sa présence et ses imports relèvent de `config`, le collecteur de `components` ne le re-parcourt pas (pas de double comptage, scellé par un test noir).
