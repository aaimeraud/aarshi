---
labels: [wayfinder:implement]
parent: 0014-map-implementation-status.md
blocked-by: []
status: open
---

# Amont : écriture étendue d'`aarshi.toml`

## Tranche

L'amont exigé par la spec ([invariants config/outillage](0008-grilling-invariants-config.md)) : `setup` écrit désormais dans `aarshi.toml` les clés `ui`, `icons`, `cache_components` et `site_url` en plus de `languages` et `package_manager`, et `add-lang`, qui réécrit le fichier en entier, **préserve** ces clés. Les nouvelles clés sont validées comme les existantes (ensembles connus, littéraux actuels conservés). Le schéma strict reste tolérant à leur absence : un projet généré avant cette extension parse sans erreur — c'est ce qui rendra `config.project-file-incomplete` un `todo` et non un arrêt.

Cette tranche **clôt le grilling [0015](0015-grilling-perimetre-implementation.md)** : sa résolution est rédigée avec le code, y compris la réponse au point « autres travaux amont » (aucun autre travail amont si l'inventaire des fichiers de base se révèle complet).

## Critères d'acceptance

- [ ] Un projet créé par cette version de `setup` porte les six clés ; un test noir le vérifie.
- [ ] `add-lang` ajoute une langue sans perdre `ui`, `icons`, `cache_components`, `site_url` ; test noir.
- [ ] Un `aarshi.toml` à l'ancien format (la seule présence de `languages` suffit : `package_manager` est déjà optionnel) parse sans erreur ; test noir.
- [ ] La résolution de 0015 est rédigée et le ticket passe à `closed`.
