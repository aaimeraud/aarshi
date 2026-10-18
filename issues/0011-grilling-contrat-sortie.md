---
labels: [wayfinder:grilling]
parent: 0005-map-status.md
blocked-by: [0006-grilling-invariants-i18n.md, 0007-grilling-invariants-pages.md, 0008-grilling-invariants-config.md, 0013-grilling-dimensions.md]
status: closed
assignee: aaimeraud
---

# Contrat de sortie : rendu humain, `--json`, codes de sortie

## Question

Une fois le catalogue d'invariants fixé (tickets « catalogue »), quel est le contrat de sortie exact de `status` ?

**Rendu humain** :

- Structure exacte du rapport groupé par dimension (titre de dimension, items, statut par dimension), en s'appuyant sur `src/output.rs` (`error`, `warning`, `success`, `info`).
- Ligne de résumé / verdict final (par ex. « projet conforme » vs « N erreurs, M à faire »), et le cas « rien à signaler ».
- Affichage des invariants **OK** : listés ou seulement comptés ?

**Codes de sortie** :

- Confirmer : non-zéro si et seulement s'il y a au moins une `error` ; des `todo` seuls → code 0. Quel code exact pour l'échec (1 ?) ?
- Faut-il un code dédié à l'erreur interne/config (`aarshi.toml` illisible), distinct de l'échec de diagnostic (à la manière d'ESLint `2`) ?

**Sortie machine `--json`** :

- Schéma exact : champs par dimension et par item (gravité, identifiant/famille de règle, `file`, `line` ?, clé de dictionnaire, locale, message), présence d'un champ de version de schéma et d'un résumé global.
- Stabilité : le schéma est-il un contrat public (à documenter et versionner) ou un détail interne ?
- Interaction avec `print_header` (`AARSHI_NO_HEADER`), les couleurs et le TTY en mode `--json` (JSON strict sur stdout ?).
- Impact dépendance : introduire `serde_json` dans `Cargo.toml`.
- Sortie sous filtre et mode court : avec `--dimension <nom>`, le code de sortie reflète-t-il encore les `error` des dimensions masquées ? Le `--json` ne contient-il que les dimensions sélectionnées ? `--errors-only` change-t-il le code de sortie ou le contenu du JSON ?

S'appuie sur [Recherche : comment les CLI conçoivent une commande de diagnostic (`doctor`/`status`)](0009-research-doctor-cli.md) : code dédié à l'erreur interne (2 ?) ; `format_version` versionné ; identifiant de règle stable ; JSON strict sur stdout ; SARIF plutôt hors v1.

## Resolution

**Rendu humain.** Une ligne par dimension, dans l'ordre canonique (`config`, `i18n`, `pages`, `sections`, `components`, `dependencies`) ; `✓` vert sur la ligne quand la dimension ne porte aucun constat ; constats **indentés sous** leur dimension, préfixés par `error:` (rouge gras) ou `todo:` (jaune gras). Le vocabulaire humain reprend **mot pour mot** les identifiants de gravité du JSON : ajouter un helper `todo()` à `src/output.rs`, symétrique de `warning()`. Les couleurs viennent de `colored`, déjà utilisé par `output.rs` ; `print_header()` reste affiché en mode humain (et reste supprimable par `AARSHI_NO_HEADER`). Les invariants OK ne sont **pas** listés individuellement : le `✓` atteste du passage sur la dimension.

```
config ✓
i18n
  error: next.config.ts redirects "/" to "/de", expected "/fr"
  todo: fr is missing a translation for `home.title`
pages ✓
sections ✓
components
  todo: components/icons/logo.svg has no generated component
dependencies ✓

error: 1 error, 2 to do
```

**Verdict final** (dernière ligne) :

- aucune `error`, aucun `todo` : `✓ project is conformant` ;
- aucune `error`, des `todo` : `✓ project is conformant (2 to do)` ;
- au moins une `error` : `error: 3 errors, 2 to do`.

**Ordre des constats dans une dimension** : par `location.file` (ordre lexicographique), puis par identifiant de règle. Un constat sans fichier se range en tête (clé de tri vide). L'ordre ne dépend pas de l'ordre d'exécution des contrôles : la sortie est stable pour un projet donné.

**Codes de sortie** :

| Code | Sens |
|---|---|
| `0` | aucune `error` — des `todo` seuls laissent `0` |
| `1` | au moins une `error` |
| `2` | `status` ne peut pas analyser : `aarshi.toml` absent, illisible ou invalide (toutes les attentes — langues, package manager, options — en dérivent). Message sur stderr. |

Convention reprise de la recherche (à la manière d'ESLint) : le code `2` distingue « je n'ai pas pu analyser » de « j'ai analysé et trouvé des erreurs ».

**Sortie machine `--json`.**

Document unique sur stdout, sans bannière ni couleur (indépendamment d'`AARSHI_NO_HEADER`) :

```json
{
  "format_version": 1,
  "summary": { "errors": 1, "todos": 2, "conformant": false },
  "dimensions": [
    {
      "id": "config",
      "errors": 0,
      "todos": 0,
      "findings": []
    },
    {
      "id": "i18n",
      "errors": 1,
      "todos": 1,
      "findings": [
        {
          "rule": "i18n.dictionary-missing",
          "severity": "error",
          "summary": "no dictionary for `fr`",
          "detail": "expected lib/i18n/dictionaries/fr.ts",
          "location": { "file": "lib/i18n/dictionaries/fr.ts", "locale": "fr" }
        }
      ]
    }
  ]
}
```

- `format_version` : entier, commence à `1`, incrémenté à chaque changement **incompatible** ; le schéma est un **contrat public**, documenté par la spec livrée.
- `summary` porte **toujours le projet entier** (`conformant` = `errors == 0`).
- `dimensions[]` contient **toujours les six** dimensions, dans l'ordre canonique, chacune avec ses compteurs et ses `findings` (éventuellement vide).
- Identifiants : `id` (dimension) et `rule` sont **stables** ; `severity` ∈ {`error`, `todo`} ; `summary` est la phrase d'une ligne montrée à l'humain ; `detail` est un complément optionnel (valeur attendue, cause) ; `location` est optionnel et ne porte que les clés pertinentes (`file`, `locale`, `key`). `line` est **réservé**, absent en v1 (les contrôles sont textuels, pas positionnels).
- En cas d'arrêt : **rien sur stdout**, le message `error:` sur stderr, code `2` — un `status --json | jq` ne casse jamais.
- Dépendance : `serde_json` (seul ajout prévu à `Cargo.toml`).

**Filtre et mode court.** `--dimension <nom>` et `--errors-only` n'agissent que sur **l'affichage** :

- le code de sortie et `summary` sont **toujours** calculés sur le projet entier : une `error` masquée par le filtre fait quand même échouer `status` ;
- le rendu humain ajoute `info: N findings hidden by --dimension` quand le filtre masque au moins un constat ;
- en `--json`, `dimensions[]` ne contient que les dimensions sélectionnées (dans l'ordre canonique), mais `summary` reste global ;
- `--errors-only` retire les constats `todo` du rendu humain et des `findings` JSON, sans changer le code de sortie ni les compteurs de `summary`.

**Identifiants de règle.** Schéma `<dimension>.<sujet>-<problème>` : le premier segment est toujours l'identifiant de dimension, ce qui garantit le regroupement et l'unicité. Liste **exhaustive** correspondant aux trois catalogues (une règle par contrôle, jamais une par fichier : le fichier concerné est dans `location`) :

| Identifiant de règle | Gravité | Objet |
|---|---|---|
| `config.file-missing` | `error` | fichier ou dossier de base absent (`app/[locale]/layout.tsx`, `app/sitemap.ts`, `app/robots.ts`, `lib/site.ts`, `lib/i18n/config.ts`, `lib/i18n/index.ts`, `components/language-switcher.tsx`, `tsconfig.json`, `eslint.config.mjs`, `postcss.config.mjs`, `next.config.ts`, `components/custom/`, `components/icons/`) |
| `config.marker-missing` | `error` | `// aarshi:pages:end` absent de `app/sitemap.ts` |
| `config.next-config-redirect` | `error` | redirect `"/"` de `next.config.ts` incohérent avec `languages[0]` |
| `config.project-file-incomplete` | `todo` | clé persistée absente d'`aarshi.toml` |
| `config.option-cache-components` | `error` | `cache_components` ↔ `cacheComponents` |
| `config.option-site-url` | `error` | `site_url` ↔ `SITE_URL` de `lib/site.ts` |
| `config.option-ui` | `error` | `ui` ↔ dépendances et composants d'UI |
| `config.option-icons` | `error` | `icons` ↔ `lucide-react` |
| `config.import-broken` | `error` | import cassé dans un fichier de base |
| `i18n.languages-mismatch` | `error` | `languages` de `config.ts` ↔ `aarshi.toml` |
| `i18n.default-locale-mismatch` | `error` | `defaultLocale` ≠ `languages[0]` |
| `i18n.direction-mismatch` | `error` | table `direction` incomplète ou incorrecte |
| `i18n.index-mismatch` | `error` | `lib/i18n/index.ts` désynchronisé |
| `i18n.dictionary-missing` | `error` | dictionnaire absent pour une langue déclarée |
| `i18n.key-missing` | `error` | clé absente par rapport au dictionnaire par défaut |
| `i18n.key-duplicate` | `error` | clé dupliquée dans un dictionnaire |
| `i18n.key-empty` | `todo` | valeur vide après `trim()` |
| `i18n.key-extra` | `todo` | clé absente du dictionnaire par défaut |
| `i18n.dictionary-orphan` | `todo` | dictionnaire pour une langue hors `aarshi.toml` |
| `pages.route-missing` | `error` | vue sans route |
| `pages.route-orphan` | `error` | route sans vue |
| `pages.title-key-missing` | `error` | clé `"<page>.title"` absente d'un dictionnaire |
| `pages.sitemap-entry-missing` | `error` | page absente de `app/sitemap.ts` |
| `pages.sitemap-entry-orphan` | `error` | entrée de sitemap sans page |
| `pages.view-import-missing` | `error` | import de la vue absent ou incorrect dans la route |
| `pages.import-broken` | `error` | import cassé dans une vue ou une route |
| `sections.file-missing` | `error` | import pendant (vue → `sections/<section>.tsx` absent) |
| `sections.title-key-missing` | `error` | clé `"<page>.<section>.title"` absente d'un dictionnaire |
| `sections.marker-missing` | `error` | `// aarshi:imports:end` ou `{/* aarshi:sections:end */}` absent de la vue |
| `sections.orphan` | `todo` | section jamais importée |
| `sections.import-broken` | `error` | import cassé dans une section |
| `components.icon-unconverted` | `todo` | `.svg` sans composant généré |
| `components.import-broken` | `error` | import cassé dans `components/` |
| `dependencies.package-missing` | `error` | paquet importé non déclaré (`@sglara/cn`, `next`, `react`, `@base-ui/react`, `lucide-react`, …) |
| `dependencies.lockfile-mismatch` | `error` | lockfile incohérent avec `package_manager` |
| `dependencies.modules-missing` | `todo` | `node_modules/` absent |

**Mise en œuvre** : ajouter `todo()` à `src/output.rs` ; ajouter `serde_json` à `Cargo.toml` ; ne pas appeler `print_header()` en mode `--json`.
