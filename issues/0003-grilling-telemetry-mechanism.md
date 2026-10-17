---
labels: [wayfinder:grilling]
parent: 0001-map.md
blocked-by: [0002-research-next-telemetry-local.md]
status: closed
---

# Télémétrie : quel mécanisme pour les projets générés ?

## Question

La recherche (ticket 0002) établit que `next telemetry disable` écrit une préférence globale à la machine, et que `NEXT_TELEMETRY_DISABLED` ne se lit qu'au niveau du processus. Quel mécanisme `aarshi` adopte-t-il pour que la télémétrie soit désactivée par défaut dans les projets qu'il génère ?

Options :
1. Variable `NEXT_TELEMETRY_DISABLED=1` exportée uniquement pendant la création (`create-next-app`). Aucune trace globale. Le projet, lui, reste sans réglage après création.
2. Variable exportée pendant la création, plus une ligne `NEXT_TELEMETRY_DISABLED=1` dans le `.env` du projet généré. Le réglage persiste pour `next dev` et `next build` (à vérifier : Next charge les `.env` avant la télémétrie).
3. `next telemetry disable` exécuté une fois, globalement sur la machine. Simple, mais touche à tous les projets.

Recommandation : option 2.

## Resolution

Option 2 retenue. Variable `NEXT_TELEMETRY_DISABLED=1` exportée pendant `create-next-app`, puis ligne `NEXT_TELEMETRY_DISABLED=1` dans le `.env` du projet généré. Aucune modification de la config globale.

À vérifier à l'implémentation : que Next charge bien le `.env` avant de lire la télémétrie (voir ticket 0002, point incertain).
