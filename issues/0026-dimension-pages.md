---
labels: [wayfinder:implement]
parent: 0014-map-implementation-status.md
blocked-by: [0023-socle-tracer-i18n.md]
status: open
---

# Dimension `pages` : vues ↔ routes, sitemap, import de vue — et primitives partagées

## Tranche

Remplir la dimension `pages` (catalogue [0007](0007-grilling-invariants-pages.md), identifiants [0011](0011-grilling-contrat-sortie.md)) : `pages.route-missing` (vue sans route), `pages.route-orphan` (route sans vue, aujourd'hui invisible), `pages.title-key-missing` (clé `"<page>.title"` absente d'un dictionnaire — l'attente vient du système de fichiers), `pages.sitemap-entry-missing`, `pages.sitemap-entry-orphan` (`"/"` = home exclu : codé en dur dans le template), `pages.view-import-missing` (import `@/views/<page>/<page>.view` absent ou incorrect dans la route), `pages.import-broken` (import cassé dans une vue ou une route).

Cette tranche pose aussi les **primitives de lecture partagées** que les dimensions suivantes réutilisent : scan **récursif** des routes sous `app/[locale]/` (toute `page.tsx`), extraction d'imports, lecture des entrées de sitemap (marqueurs de `app/sitemap.ts`), inventaire des vues et sections. Ce qui se reprend de `src/add.rs` / `src/check.rs` et ce qui s'introduit est décidé ici et consigné dans la résolution de [0016](0016-grilling-moteur-status.md) (son point « primitives de lecture partagées », élargie si besoin).

## Critères d'acceptance

- [ ] Chaque règle a au moins un test noir (constat déclenché sur fixture, `rule`, gravité, message).
- [ ] Vue ↔ route fonctionne dans les deux sens : vue sans route et route orpheline (fichier ajouté à la main) détectées.
- [ ] Les primitives sont réutilisables par les autres dimensions (extraction d'imports, sitemap, inventaire) et documentées dans la résolution du moteur.
- [ ] Un projet sans aucun dossier `views/` produit des constats sans jamais planter (règle transverse).
