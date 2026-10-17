---
labels: [wayfinder:research]
parent: 0001-map.md
blocked-by: []
status: closed
---

# Recherche : `next telemetry disable` accepte-t-elle un réglage par projet ?

## Question

La commande `next telemetry disable` peut-elle être limitée à un projet (et pas seulement à la configuration globale de la machine) ? Quelle est la forme exacte de la variable d'environnement `NEXT_TELEMETRY_DISABLED` et à quel moment elle doit être définie pour que la création via `create-next-app` ne télémétrie pas ?

Cette réponse conditionne la décision « télémétrie désactivée par projet » de la carte.

## Resolution

**Réponse directe**

- `next telemetry disable` n'est pas limitée à un projet. Elle écrit une préférence dans une configuration globale à la machine (fichier `conf` nommé `nextjs`, chemin affiché par la commande). Le `cwd` ne sert qu'en CI ou sous Docker, où le stockage passe dans `<projet>/.next/cache`. Ce n'est pas une option documentée par projet.
- Variable d'environnement : `NEXT_TELEMETRY_DISABLED=1`. La doc recommande `1`, mais le code teste seulement la présence d'une valeur non vide (`!!process.env.NEXT_TELEMETRY_DISABLED`). Donc `0` désactive aussi, et une chaîne vide n'a aucun effet.
- Pour `create-next-app` : `create-next-app` lui-même n'envoie pas de télémétrie Next. Il lance l'installation des paquets et `runTypegen` dans le nouveau projet. Il faut donc exporter la variable dans le shell **avant** la commande (`NEXT_TELEMETRY_DISABLED=1 npx create-next-app@latest ...`), car le projet n'existe pas encore et un `.env` du projet n'est pas chargé.

**Sources**

- Doc officielle télémétrie : https://nextjs.org/telemetry (commande `next telemetry disable` et variable `NEXT_TELEMETRY_DISABLED=1`)
- Code `telemetry/storage.ts` (lecture de `NEXT_TELEMETRY_DISABLED`, stockage `conf`, `cwd` seulement en CI/Docker) : https://github.com/vercel/next.js/blob/canary/packages/next/src/telemetry/storage.ts
- Code `cli/next-telemetry.ts` (écriture globale via `setEnabled`) : https://github.com/vercel/next.js/blob/canary/packages/next/src/cli/next-telemetry.ts
- Code `create-next-app/index.ts` et `create-app.ts` (install, `runTypegen`, pas de télémétrie Next) : https://github.com/vercel/next.js/blob/canary/packages/create-next-app/create-app.ts

**Points incertains**

- Le chemin exact du fichier de configuration sur macOS n'a pas été vérifié. La commande l'affiche (`Your preference has been saved to ...`).
- Je n'ai pas vérifié si `next typegen` (lancé par `runTypegen`) émet un événement de télémétrie. Dans le doute, la variable exportée avant la commande couvre ce cas.
- Les URLs GitHub pointent vers `canary` ; les versions stables peuvent différer. Les pages `docs/architecture/telemetry` n'existent plus (404), la page `nextjs.org/telemetry` a été utilisée à la place.
- Je n'ai pas vérifié le comportement de `next telemetry disable` par projet dans les versions stables récentes. Elle reste globale à la date de la lecture du code.
