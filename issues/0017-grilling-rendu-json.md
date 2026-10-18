---
labels: [wayfinder:grilling]
parent: 0014-map-implementation-status.md
blocked-by: [0016-grilling-moteur-status.md]
status: closed
assignee: aaimeraud
---

# Rendu humain, `--json` et wording des messages

## Question

Le [contrat de sortie](0011-grilling-contrat-sortie.md) fixe la forme ; ce ticket fixe sa **mise en œuvre** :

- **Rendu humain.** Une ligne par dimension dans l'ordre canonique, `✓` vert quand la dimension est propre, constats indentés préfixés `error:` (rouge gras) ou `todo:` (jaune gras), sur `src/output.rs`. Ajout du helper `todo()` symétrique de `warning()`. Cas « rien à signaler » et ligne de verdict (trois formes, mise au pluriel). Ligne `info: N findings hidden by --dimension` quand le filtre masque un constat. `print_header()` reste affiché en mode humain (`AARSHI_NO_HEADER`).
- **`--json`.** Structs de sérialisation (`format_version`, `summary`, `dimensions[]`, `findings[]`), ordre des clés, champs optionnels omis quand absents, compteurs par dimension. JSON strict sur stdout, jamais de bannière ni de couleur ; rien sur stdout en cas d'arrêt. Ajout de `serde_json` à `Cargo.toml`.
- **Wording.** Convention de rédaction des 36 phrases `summary` en anglais (et du champ `detail` : valeur attendue, cause), avec la grammaire des `location`. La spec laisse le texte à l'implémentation : il est écrit ici et figé par les tests. Produit-on la liste complète des libellés, ou seulement les conventions de rédaction ?

## Resolution

**Emplacement et structure.** Le rendu vit dans `src/status/render.rs` : `human(report, options)` imprime le rapport, `json(report, options) -> String` sérialise le document. `status::run` choisit : mode `--json` imprime **seulement** le document (jamais de bannière, jamais de couleur — les compteurs passent dans les clés du JSON, pas par `output.rs`) ; le mode humain affiche `print_header()` d'abord, et `status` gère lui-même l'en-tête (voir [0018](0018-grilling-codes-sortie-alias-check.md)) pour garantir « rien sur stdout » dans les autres cas.

**Rendu humain.** Un `println!` par ligne, couleurs via `colored` déjà utilisé par `src/output.rs` :

- une ligne par dimension dans l'ordre canonique (`Dimension::ALL`) ; `✓` vert posé à la suite de l'identifiant quand la dimension ne porte **aucun** constat dans le rapport complet (avec `--errors-only`, une dimension qui n'a que des `todo` cachés affiche son identifiant nu : elle n'est pas propre non plus) ;
- constats **indentés de deux espaces** sous leur dimension, préfixés `error:` (rouge gras, `output::error`) ou `todo:` (jaune gras, nouveau helper `output::todo`, symétrique de `warning()`) ; seule la phrase `summary` est montrée à l'humain, le `detail` reste une affaire de JSON ;
- verdict final précédé d'une ligne vide, calculé sur le **projet entier** : `✓ project is conformant` / `✓ project is conformant (2 to do)` / `error: 1 error, 2 to do` ;
- après le verdict, `info: N findings hidden by --dimension` quand le filtre `--dimension` masque au moins un constat (`N` = constats des dimensions non sélectionnées ; `--errors-only` ne déclenche jamais cette ligne, la règle 0011 ne la prévoit que sous filtre de dimension).

Verdicts et pluriels tiennent en un helper `amount(n, singulier, pluriel)` : `1 error`/`2 errors`, et `to do` garde une forme fixe (`1 to do`, `2 to do`), conformément à l'exemple de [0011](0011-grilling-contrat-sortie.md).

**`--json`.** `serde_json` ajouté à `Cargo.toml` (seule dépendance nouvelle). Structs de sérialisation dédiés (`JsonDoc`/`JsonSummary`/`JsonDimension`) dont l'ordre de déclaration **est** l'ordre des clés : `format_version` (`1`), `summary` (`errors`, `todos`, `conformant` — projet entier, jamais filtré), `dimensions[]` (`id`, `errors`, `todos`, `findings[]` — les six dimensions ou seulement les sélectionnées, toujours dans l'ordre canonique). Chaque finding sérialise `rule`, `severity`, `summary`, puis `detail` et `location` omis quand absents ; `location` ne sérialise que `file`/`locale`/`key` selon les clés utiles. `--errors-only` retire les `todo` des `findings` humain et JSON, sans toucher aux compteurs ni au `summary`. Le document est imprimé en JSON indenté (`to_string_pretty`) : lisible et diffable, et le contrat ne fixe pas la densité — la seule exigence est un document unique valide, strict même quand `AARSHI_NO_HEADER` est vide, testé.

**Conventions de wording** (les quatre phrases du tracer sont figées par les tests ; les tickets suivants appliquent les mêmes règles et figent les leurs) :

- `summary` : phrase anglaise d'une ligne, sans point final, sans majuscule de titre ; elle nomme **l'objet en premier** et l'énonce avec des accents graves autour des identifiants. Le lexique fixé par le tracer : `no dictionary for \`de\`` ; `lib/i18n/config.ts lists [...], but aarshi.toml lists [...]` ; et la grammaire des clés `` `<locale>` `<verbe> ... for \`<cle>\`` `` : `en is missing a translation for \`home.title\``, `fr has an empty translation for \`home.intro\``.
- `detail` : complément nu, sans point final — la valeur attendue ou la cause : `expected lib/i18n/dictionaries/de.ts` (fichier attendu), `expected ["fr", "de"]` (liste attendue), `reference dictionary \`fr\`` (référence des comparaisons de clés).
- `location` : `file` est le chemin relatif à la racine du projet en slashes (`lib/i18n/dictionaries/en.ts`) ; `locale` et `key` reprennent le code de langue et la clé exacte du constat.
