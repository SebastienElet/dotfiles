# Registre d’invariants du harnais — conception

## Contexte

L’issue GitHub 159 demande de relier durablement les constats de revue aux invariants du harnais et
aux oracles qui les prouvent. Son hypothèse sur `pr-feedback` n’est plus actuelle : la PR 175 a
volontairement transformé cette commande en collecteur factuel, puis la PR 254 a conservé cette
frontière lors de sa migration en skill.

Le flux retenu préserve cette décision plus récente :

```text
pr-feedback factuel
  → harness-reflection classe et rapproche
  → candidat nommé dans le registre
  → approbation humaine explicite
  → surface choisie et oracle de régression
  → mesure, vérification ou retraite
```

## Objectif

Ajouter une source versionnée et vérifiable qui relie chaque invariant personnel promu à ses
preuves de revue, son périmètre, sa surface d’application, son oracle et son état, sans permettre à
un agent de promouvoir seul une préférence isolée ni étendre la topologie gérée par Arnes.

## Autorités et frontières

- `pr-feedback` reste le producteur factuel en lecture seule. Il conserve les occurrences, les URL
  ou références immuables, l’impact observé et les incertitudes sans classifier la cause ni proposer
  une mutation du harnais.
- `harness-reflection` devient le propriétaire aval de la classification, du rapprochement avec un
  invariant existant, du candidat falsifiable et de la demande d’approbation.
- `harness/invariants/registry.json` devient l’unique source versionnée des invariants personnels.
- Un validateur TypeScript avec Zod vérifie la structure et les transitions représentées dans le
  registre. Il ne modifie jamais le registre et ne constitue pas une approbation.
- Git conserve l’historique et la revue des mutations. Arnes continue seulement à distribuer le
  skill existant ; son manifeste et ses adaptateurs ne sont pas étendus.

Cette répartition respecte les ADR 024, 038 et 040 sur la source commune et les frontières du
harnais, l’ADR 036 sur l’admission comportementale par mesure et l’ADR 041 sur TypeScript pour le
parsing et la politique. `agent-memory` reste hors périmètre conformément à l’ADR 042 : une mémoire
hors Git n’est ni une autorité de promotion ni un registre versionné.

## Modèle canonique

Le document racine porte une version de schéma et une liste d’invariants. Un invariant utilise un
identifiant sémantique stable et sépare trois axes que l’issue mélangeait :

- `lifecycle` : `candidate`, `active` ou `retired` ;
- `controlKind` : `probabilistic` ou `enforceable` ;
- `verification` : `unverified`, `measured` ou `verified`.

Chaque invariant enregistre également :

- une formulation actionnable et une classe de cause parmi `not-applied`, `not-loaded`, `unknown`,
  `blind-spot` et `judgment` ;
- une sévérité et des sources de revue dans une union fermée GitHub/Bitbucket Cloud ; chaque
  occurrence sépare l’identité canonique de la PR de l’URL immuable du commentaire ou de la revue ;
  Bitbucket Cloud accepte le lien HTML de commentaire `/_/diff#comment-ID` exposé par l’API ;
- un périmètre `cross-project` ou `project-local`, avec des exceptions structurées et justifiées ;
- une surface parmi instruction toujours chargée, skill conditionnel, hook, permission, lint, type,
  test architectural ou contrat local ;
- pour une surface exécutoire, un oracle nommé qui cible le chemin d’échec, un fichier de test
  régulier suivi par Git et découvert par la suite, ainsi que son invocation exacte ;
- pour chaque agent `claude`, `codex` et `cursor`, un statut `supported` ou `unsupported`, son
  mécanisme ou sa raison, et le dernier environnement vérifié lorsqu’il existe ;
- pour un invariant retiré, la date, la raison et éventuellement l’identifiant qui le remplace.

Les surfaces d’instruction et les skills sont probabilistes. Hooks, permissions, lint, types et
tests architecturaux sont exécutoires seulement si le mécanisme choisi refuse effectivement le
chemin dangereux ; un contrat local en prose reste probabiliste.

## Invariants du validateur

Le parseur rejette les valeurs inconnues au lieu de leur donner une valeur plausible. Après le
parsing Zod, les règles sémantiques refusent notamment :

- deux invariants portant le même identifiant ou réutilisant la même occurrence de preuve ;
- une promotion `active` sans approbation humaine explicite ;
- une promotion ordinaire sans sources provenant d’au moins deux PR distinctes ;
- une promotion sur une seule PR sans sévérité forte établie ;
- la promotion d’un `judgment` en contrôle ;
- une surface et un `controlKind` incompatibles ;
- un contrôle exécutoire actif ou vérifié sans oracle nommé, sans fichier de test régulier suivi et
  découvert ou, lorsque `verification.state` vaut `verified`, avec une invocation différente de
  celle mesurée ;
- un état `verified` sans mesure verte, environnement et date ;
- un invariant `retired` sans raison et date, une retraite sur un autre cycle de vie, ou un graphe
  de remplacement cyclique ou à cible inconnue ;
- une déclaration de consommation qui omet un des trois agents ou présente un comportement non
  supporté comme supporté.

Un candidat peut rester incomplet tant que ses inconnues sont explicites. Le seuil de deux PR ou de
sévérité forte autorise la proposition de promotion ; il ne remplace ni l’approbation humaine ni la
preuve de l’oracle.

## Workflow de `harness-reflection`

À partir d’un rapport `pr-feedback`, le skill :

1. vérifie les preuves et classe la cause sans modifier le rapport source ;
2. recherche d’abord un invariant existant par identifiant, formulation et sources ;
3. propose soit l’ajout d’une occurrence à cet invariant, soit un unique nouveau candidat ;
4. conserve le candidat dans la session jusqu’à approbation explicite ;
5. après approbation, prépare la surface et le registre, valide d’abord les deux — le registre via la CLI
   sur une copie temporaire — puis les applique comme un changement cohérent ; toute erreur restaure
   tous les fichiers touchés ;
6. exige le test du chemin d’échec avant tout contrôle exécutoire ;
7. ne passe à `verified` qu’avec une exécution verte nommée et son environnement ;
8. retire un invariant en conservant son historique, ses exceptions et son remplacement éventuel,
   avec les mêmes copies préparées, validation préalable par la CLI, application cohérente et
   compensation de tous les fichiers touchés.

La recherche préalable et l’unicité des sources empêchent une répétition de créer silencieusement
un second invariant. Les changements de skills et d’instructions continuent d’être routés vers
`skill-manager` et `agent-instructions`.

## Entrée et erreurs

Le point d’entrée supporté est une commande de validation en lecture seule ciblant par défaut le
registre canonique. Il accepte un chemin explicite pour les fixtures, parse le JSON une fois à la
frontière, produit des diagnostics stables avec le chemin du champ concerné et retourne un statut
non nul pour tout fichier absent, JSON invalide, schéma inconnu, règle sémantique violée ou oracle
non régulier, non suivi, non découvert ou introuvable. L’inspection locale refuse le lien symbolique
final, ouvre le fichier sans le suivre, compare son identité avant et après les sondes, confine son
chemin réel et exige un mode de fichier régulier dans l’index Git ; la politique pure reçoit ce
résultat par injection.

Il ne télécharge aucune preuve, n’exécute aucun oracle et ne réécrit jamais le JSON. Les références
distantes sont des identifiants traçables ; leur disponibilité réseau n’est pas confondue avec la
validité locale du registre.

## Tests et preuves historiques

Les tests Bun invoquent le module réel et le point d’entrée livré. Ils couvrent au minimum :

- parsing valide et erreurs Zod lisibles ;
- seuil de deux PR, exception de sévérité forte et refus d’une occurrence isolée ordinaire ;
- déduplication globale d’une occurrence de preuve, sans confondre plusieurs commentaires d’une PR ;
- distinction probabiliste/exécutoire ;
- refus d’un contrôle exécutoire sans oracle de chemin d’échec ;
- vérification avec et sans mesure verte et environnement ;
- exceptions, retraite discriminée et graphe de remplacement acyclique ;
- présence séparée de Claude, Codex et Cursor, y compris un cas `unsupported` ;
- fichier absent, JSON invalide, version inconnue, chemin d’oracle absent, lien symbolique final,
  mode d’index non régulier et substitution pendant les sondes.

Deux fixtures historiques exercent le flux sans inscrire automatiquement un nouvel invariant réel :

- PR 206 : une URL de fetch rejetée copiait des credentials dans `stderr`, cas de sévérité forte
  avec oracle de non-divulgation ;
- PR 207 : un décodage UTF-8 permissif transformait des octets invalides et permettait un succès ou
  une erreur aval trompeuse, cas de frontière qui doit échouer explicitement.

Les gates attendues sont `bun test`, `tsc --noEmit`, le lint et le formatage TypeScript, le contrôle
statique du skill, CSpell sur les nouveaux textes et la vérification des projections existantes. La
suite complète est exercée localement sur macOS. Le workflow TypeScript actuel de GitHub Actions
l’exerce sur Ubuntu seulement ; aucune preuve hébergée macOS n’est revendiquée pour cette suite.

## Non-objectifs

- Modifier le contrat factuel de `pr-feedback`.
- Ingérer automatiquement les commentaires GitHub.
- Exécuter, promouvoir ou retirer automatiquement un invariant.
- Étendre Arnes, son manifeste ou ses adaptateurs.
- Remplacer les contrats d’architecture ou de domaine propres à un dépôt.
- Faire d’une préférence stylistique isolée un contrôle bloquant.

## Critères d’acceptation

- Le registre JSON versionné est l’unique source des invariants personnels nommés.
- `harness-reflection` consomme le relevé factuel sans reconstruire l’historique de revue.
- Le seuil deux PR ou sévérité forte et tous les refus ci-dessus sont prouvés par des tests du vrai
  validateur.
- Aucun contrôle exécutoire actif ou vérifié n’existe sans oracle de chemin d’échec versionné,
  suivi et découvert ; lorsque `verification.state` vaut `verified`, l’oracle est lié exactement à
  son invocation mesurée.
- Une occurrence de preuve répétée est refusée globalement ; plusieurs occurrences distinctes de la
  même PR restent autorisées et ne comptent que pour une PR.
- Exceptions et retraite conservent l’historique au lieu de supprimer la trace.
- Claude, Codex et Cursor sont déclarés séparément, y compris lorsque l’un est non supporté.
- Les deux constats historiques traversent le flux dans des fixtures sans devenir des règles réelles
  par simple présence dans les tests.
