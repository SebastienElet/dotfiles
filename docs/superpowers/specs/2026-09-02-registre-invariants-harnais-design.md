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
  → surface proposée, manifeste exact et oracle de régression
  → approbation fournie par le contexte humain, identité non authentifiée
  → application par l’outil propriétaire de la surface
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
- Le champ `approval` consigne une attestation transmise après l’approbation fournie dans le contexte
  humain. Ni le registre, ni la CLI, ni l’entrée du workflow n’authentifient son origine ou l’identité
  de `approvedBy`, et ce champ n’est pas une preuve autonome. Le skill interdit procéduralement à
  l’agent de fabriquer cette attestation. Le changement du routeur et de sa référence a remis
  l’artefact comportemental à `pending` avec `runs: []`. Il ne prouve donc actuellement aucun chemin
  comportemental ; les replays antérieurs ne se transfèrent pas au nouveau digest.
- Git conserve l’historique et la revue des mutations. Arnes continue seulement à distribuer le
  skill existant ; son manifeste et ses adaptateurs ne sont pas étendus.

Cette répartition respecte les ADR 024, 025, 038 et 040 sur la source commune, les instructions
projet et les frontières du harnais, l’ADR 036 sur l’admission comportementale par mesure et l’ADR
041 sur TypeScript pour le
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
  GitHub accepte aussi l’ancre inline `#discussion-diff-ID` ; Bitbucket Cloud accepte le lien HTML
  de commentaire `/_/diff#comment-ID` exposé par l’API ;
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
- une promotion `active` sans attestation d’approbation enregistrée ;
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
  supporté comme supporté ;
- un `conditional-skill` actif dont `candidateTextExact` diffère de son `statement` canonique.

Un candidat peut rester incomplet tant que ses inconnues sont explicites. Le seuil de deux PR ou de
sévérité forte autorise la proposition de promotion ; il ne remplace ni l’approbation fournie par
le contexte humain ni la preuve de l’oracle. Le validateur structurel ne sait pas authentifier cette
approbation.

## Workflow de `harness-reflection`

À partir d’un rapport `pr-feedback`, le skill :

1. vérifie les preuves et classe la cause sans modifier le rapport source ;
2. recherche d’abord un invariant existant par identifiant, formulation et sources ;
3. propose soit l’ajout d’une occurrence à cet invariant, soit un unique nouveau candidat ; pour un
   lien, l’état courant et l’état proposé contiennent chacun exactement une occurrence cible ;
4. prépare en session le registre et, pour une surface fichier, cette surface ; il capture leurs
   préimages et construit un manifeste fermé qui contient les chemins cibles exacts, chaque contenu
   initial et remplacement exact, et le delta avant/après du seul invariant ciblé ; le type de
   mutation est dérivé de cette transition ;
5. présente ce manifeste au contexte humain avant l’approbation, sans prétendre authentifier la
   personne, puis transmet à l’API la requête structurée et l’attestation inchangées ; le code en
   vérifie la cohérence exacte sans pouvoir distinguer une origine humaine d’une fabrication ;
6. pour `conditional-skill`, conserve `SKILL.md` comme routeur fermé et immuable : son invocation lit
   directement les enregistrements actifs de cette surface dans le registre et utilise leur
   `statement` comme texte conditionnel exact, sans le dupliquer dans le skill ; le manifeste est
   donc registry-only ;
7. pour une surface fichier, route la mutation approuvée vers son outil propriétaire obligatoire :
   `agent-instructions` pour `harness/AGENTS.md` ou le contrat local exact `AGENTS.md`, et la frontière
   scripts/enforcement pour un contrôle exécutoire ; une modification réelle du routeur reste une
   modification de skill distincte, possédée par `skill-manager`, et invalide son artefact
   comportemental ; le moteur du registre ne fournit aucune API d’écriture arbitraire ;
8. exécute le doctor ou le contrat de l’outil propriétaire, puis vérifie en lecture seule que la
   surface appliquée est exactement le remplacement approuvé et que le registre courant reste sa
   préimage approuvée ; il refuse tout autre chemin, préimage, remplacement ou delta d’invariant ;
9. écrit alors uniquement le remplacement approuvé de `harness/invariants/registry.json`, puis lance
   la CLI sur le registre résultant ; Git porte la revue et la récupération d’une interruption ;
10. exige le test du chemin d’échec avant tout contrôle exécutoire ;
11. ne passe à `verified` qu’avec une exécution verte nommée et son environnement ;
12. retire un invariant seulement si tous ses champs historiques restent identiques, dont les sources
    et exceptions ; seuls `approval`, `lifecycle` et `retirement` peuvent changer, la nouvelle
    attestation devant correspondre exactement au registre et au manifeste ; l’outil propriétaire
    retire d’abord le texte exact d’une surface fichier, tandis qu’un `conditional-skill` est retiré
    uniquement dans le registre, puis le registre est mis à jour.

Le validateur borné de transition accepte un lien seulement de `candidate` vers `candidate` ou
d’`active` vers `active`. Les sources canoniques existantes restent dans leur ordre, au moins une
nouvelle occurrence distincte est ajoutée, et aucun champ métier ne change. La nouvelle attestation
exacte est le seul delta administratif admis. Une proposition de tout nouveau candidat conserve son
manifeste structurel, mais ne passe pas par cette API de transition, qui exige une cible unique avant
et après.

Ce workflow ne revendique ni transaction multi-fichier, ni atomicité, ni sérialisation face à un
autre écrivain. Une interruption entre surface et registre peut laisser un état intermédiaire ; la
préimage et le remplacement approuvés, les validations bornées et le diff Git permettent de le
détecter et de le réconcilier, sans promettre une récupération automatique.

La recherche préalable et l’unicité des sources empêchent une répétition de créer silencieusement
un second invariant. Le hash byte-exact du petit routeur est vérifié dans son contrat local afin que
toute prose contradictoire échoue ; il ne devient pas un hash global des jobs CI. Les changements
réels de skills et d’instructions continuent d’être routés vers `skill-manager` et
`agent-instructions`.

## Entrée et erreurs

Le point d’entrée supporté est une commande de validation en lecture seule ciblant par défaut le
registre canonique. Il accepte un chemin explicite pour les fixtures, parse le JSON une fois à la
frontière, produit des diagnostics stables avec le chemin du champ concerné et retourne un statut
non nul pour tout fichier absent, JSON invalide, schéma inconnu, règle sémantique violée ou oracle
non régulier, non suivi, non découvert ou introuvable. L’inspection locale refuse le lien symbolique
final, ouvre le fichier sans le suivre, compare son identité avant et après les sondes, confine son
chemin réel et exige un mode de fichier régulier dans l’index Git ; la politique pure reçoit ce
résultat par injection.

Il ne télécharge aucune preuve et ne réécrit jamais le JSON. Le parseur bibliothèque reste purement
structurel. Le point d’entrée CLI exécute l’invocation locale déclarée par chaque enregistrement
`verified` qui lie un oracle à sa dernière mesure et échoue si elle échoue ; il ne prétend rien sur
un oracle non déclaré ou non exécuté. Les références distantes sont des identifiants traçables ;
leur disponibilité réseau n’est pas confondue avec la validité locale du registre.

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

Deux fixtures historiques conservent leurs vraies URL enregistrées et exercent le parsing des
sources, la déduplication, la politique, la construction d’une proposition candidate et la
validation de son manifeste. Elles ne prouvent ni application de surface ni promotion complète :

- PR 206 : une URL de fetch rejetée copiait des credentials dans `stderr`, cas de sévérité forte
  avec oracle de non-divulgation ;
- PR 207 : un décodage UTF-8 permissif transformait des octets invalides et permettait un succès ou
  une erreur aval trompeuse, cas de frontière qui doit échouer explicitement.

Une fixture locale distincte, marquée `synthetic-local-not-historical`, porte les deux sources
synthétiques nécessaires à l’oracle d’intégration. Elle traverse proposition, manifeste, application
de fixture, validation, inscription, CLI et oracle, puis la retraite. Elle ne présente pas ses URL
`example` comme des preuves historiques et ne simule aucune authentification humaine. L’artefact
agent actuel est `pending` avec `runs: []` après le changement du routeur et ne prouve aucun
comportement courant. Les replays du digest antérieur ne sont pas attribués à cet artefact.

Les gates attendues sont `bun test`, `tsc --noEmit`, le lint et le formatage TypeScript, le contrôle
statique du skill, CSpell sur les nouveaux textes et la vérification des projections existantes. La
suite complète est exercée localement sur macOS. Le workflow TypeScript actuel de GitHub Actions
l’exerce sur Ubuntu seulement ; aucune preuve hébergée macOS n’est revendiquée pour cette suite.

## Non-objectifs

- Modifier le contrat factuel de `pr-feedback`.
- Ingérer automatiquement les commentaires GitHub.
- Promouvoir ou retirer automatiquement un invariant sans le workflow et les outils propriétaires.
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
- L’approbation enregistrée est une attestation non authentifiée ; le manifeste exact est présenté
  au contexte humain avant cette entrée, et la requête, les préimages, remplacements et le seul delta
  d’invariant ciblé doivent lui correspondre. L’interdiction d’auto-assertion est procédurale dans le
  skill. L’artefact comportemental est `pending` avec `runs: []` après changement du routeur ; aucun
  chemin comportemental courant n’est présenté comme couvert.
- Une surface est appliquée uniquement via son outil propriétaire, puis son doctor ou contrat passe
  avant l’écriture bornée du registre ; aucune API de production ne remplace arbitrairement une
  surface et aucune transaction multi-fichier n’est promise.
- Pour une surface fichier, la promotion ajoute et la retraite retire réellement
  `candidateTextExact`. Pour `conditional-skill`, ce texte égale exactement `statement` dans le
  registre et la retraite du record suffit ; `SKILL.md` ne change jamais pour porter le candidat.
  Une préimage égale au remplacement est refusée. Ces critères et le doctor propriétaire bornent la
  preuve sans prétendre décider la sémantique générale du texte.
- Pour un enregistrement `verified` dont la dernière mesure déclare une invocation, la CLI exécute
  réellement cette invocation et échoue avec elle ; le parseur bibliothèque ne l’exécute pas.
- Claude, Codex et Cursor sont déclarés séparément avec un mécanisme fermé compatible avec la surface,
  y compris lorsque l’un est non supporté.
- Les deux constats historiques atteignent honnêtement la proposition et son manifeste ; la fixture
  synthétique distincte prouve le workflow local complet sans devenir une règle réelle.
