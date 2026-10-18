---
labels: [wayfinder:implement]
parent: 0014-map-implementation-status.md
blocked-by: [0027-dimension-sections.md]
status: open
---

# Dimension `components` : icônes non converties et resolved imports

## Tranche

Remplir la dimension `components` (catalogue [0007](0007-grilling-invariants-pages.md)) :

- `components.icon-unconverted` (`todo`) : `components/icons/<name>.svg` sans `<name>.tsx` (conversion pas encore faite). `<name>.tsx` sans `.svg` est **ignoré**.
- `components.import-broken` (`error`) : import cassé dans `components/`.
- **Résolution des imports des fichiers connus** : les imports des routes, vues, sections et `app/[locale]/layout.tsx` (y compris `language-switcher`) doivent résoudre vers un fichier existant — un composant référentiel manquant est un `error`, classé dans la dimension de l'objet fautif ; un composant de `components/custom/` **non utilisé n'est pas signalé**.

Cette tranche s'appuie sur les primitives d'imports et l'inventaire des fichiers connus posés par [0026](0026-dimension-pages.md)/[0027](0027-dimension-sections.md).

## Critères d'acceptance

- [ ] Icône `.svg` sans `.tsx` → un `todo` ; `.tsx` sans `.svg` → rien ; les deux par tests noirs.
- [ ] Un import d'un composant inexistant (vues/routes/sections/layout) produit un constat `error` dans la dimension de l'objet fautif, avec tests.
- [ ] Un composant `custom` non importé ne génère **aucun** constat ; test noir.
- [ ] La présence de `components/language-switcher.tsx` reste dans la dimension `config` ([0024](0024-dimension-config.md)) — pas de double comptage.
