# USER.md — Biais et attentes de l'utilisateur

Préférences de collaboration et biais techniques stables. Identité de l'agent →
`SOUL.md` ; invariants, conventions et contrôles propres au projet → `AGENTS.md`,
rules, skills, linters, hooks ou CI.

## Acquis

Shell, git et Unix ; web moderne (JS/TS, bundlers, npm) ; infra et conteneurs
(Docker, CI/CD, réseau) ; backend et bases de données (SQL, modélisation,
transactions). Aucun rappel de fondamentaux sur ces sujets : aller au fait.

## Biais techniques

- **Simplicité, testabilité, localité.** À qualité comparable, préférer la solution
  concise, facilement testable et reconstruisible par un humain, un junior ou un agent.
  Éviter les abstractions qui imposent de traverser de nombreux fichiers pour comprendre
  un comportement simple ; plusieurs éléments cohésifs peuvent rester dans le même fichier.
- **Fonctionnel et immuable par défaut.** Préférer fonctions, composition, transformations
  explicites et valeurs immuables. Dans un framework orienté classes, garder les classes à
  sa frontière et le cœur métier fonctionnel et pur. Guard clauses et early returns sont
  préférés aux imbrications ; `map`/`filter`/`reduce` expriment un résultat, `for`/`forEach`
  des effets sans résultat.
- **Architecture actée > simplicité locale.** Respecter les frontières décidées par l'équipe.
  Une contradiction certaine avec un ADR ou invariant bloque le chemin concerné ; ne pas
  créer une exception locale pour contourner une architecture qui doit être corrigée.
- **Domaine canonique.** Centraliser les décisions et invariants métier dans le domaine quand
  ils ne peuvent pas être garantis plus bas ; handlers et adapters orchestrent dépendances
  et effets de bord. Séparer modèle DB/ORM, DTO de frontière et modèle métier dès le début.
  La frontière transactionnelle suit l'invariant : repository s'il le porte seul, handler
  s'il coordonne plusieurs repositories.
- **WET et YAGNI.** Deux chemins spécifiques valent mieux qu'une abstraction hypothétique.
  Extraire quand données, invariants et évolution probable sont réellement communs. Préférer
  des APIs spécifiques au cas d'usage ; éviter booléens positionnels et généralisation précoce.
- **SSOT.** Une information structurante a une source canonique ; validation, types, clients,
  documentation ou autres représentations en dérivent. Ne jamais éditer le généré. Maintenir
  un lexique métier canonique par projet et renommer quand il évolue pour éviter les synonymes.
- **Typage métier strict.** Faire échouer au plus tôt : typage avant test, test avant runtime.
  Rendre les états invalides non représentables avec branded types, unions discriminées et
  types fermés. Préférer des types dédiés pour dates métier, montants et identifiants ; des IDs
  de domaines différents restent incompatibles. Pour la monnaie, préférer l'unité minimale
  entière aux flottants.
- **Validation aux frontières.** Parser et valider les entrées externes une fois, puis faire
  circuler des valeurs fiables ; en TypeScript, Zod est un bon défaut. Éviter `null`/`undefined`
  ambigus. Valider la configuration au démarrage et injecter un objet typé ; `process.env`
  hors frontière est un code smell.
- **TDD par défaut.** Test avant code ; pour un bug, reproduire d'abord la régression. Exception :
  déclaratif sans comportement utile à tester, où un smoke test/CI peut suffire. Choisir l'oracle
  le moins coûteux qui prouve réellement l'invariant : unitaire si possible, intégration dès que
  la preuve appartient à la DB, une transaction, un mapping ORM, un protocole ou autre composant.
- **In-memory plutôt que mocks.** Pour les dépendances applicatives, préférer une implémentation
  in-memory minimale qui évolue sous la pression des tests. Tester le vrai composant lorsque
  l'invariant lui appartient, par exemple index unique ou trigger DB. Tester les invariants et
  comportements observables, jamais les détails d'implémentation ou du coverage pour lui-même.
- **Rien de cassé n'est toléré.** Un test intermittent est un bug : ni `retry`, ni `skip`, chercher
  la cause. Le coverage peut être un garde-fou modéré selon l'état du projet, jamais un objectif
  qui justifie des tests sans valeur.
- **DB comme verrou canonique quand possible.** Préférer contraintes déterministes en DB et
  traduire leurs erreurs en erreurs métier plutôt que maintenir un pré-check redondant. Nommer
  contraintes et index importants. Isolation tenant et autres invariants structurels doivent
  descendre au niveau le plus bas capable de les garantir.
- **ORM pour le simple, SQL typé pour le complexe.** Un accès DB dans une boucle déclenche la
  recherche d'un bulk, `JOIN`, agrégat ou chargement groupé. Corriger tôt N+1, fan-out réseau/DB
  et autres smells structurels ; mesurer les optimisations dépendantes du workload. Normaliser
  par défaut ; dénormaliser quand le ratio lecture/écriture ou une mesure le justifie.
- **Cache en dernier recours.** Optimiser d'abord schéma, index et requêtes. Introduire un cache
  seulement si charge, ratio lecture/écriture, tolérance à la fraîcheur ou SLO le justifient.
- **Migrations progressives et reprenables.** Préférer expand/contract. Découper les gros
  changements en migrations petites, atomiques, compréhensibles et rejouables autant que
  possible plutôt qu'une opération monolithique difficile à reprendre.
- **Concurrence bornée et asynchrone.** `Promise.all` convient à un ensemble indépendant de
  taille connue ; sinon limiter explicitement la concurrence. Un fan-out DB/API est d'abord
  un signal à remplacer par batch, requête groupée ou queue. Pour les systèmes externes,
  préférer queue/worker dès que le métier le permet.
- **Jobs rejouables.** Chercher l'idempotence, rendre la progression durable et arbitrer doublon
  versus omission selon leur coût métier. Retry uniquement les opérations rejouables sur erreurs
  plausiblement transitoires ; respecter `429`/`Retry-After`. Timeouts explicites et courts ;
  un traitement réellement long devrait idéalement devenir asynchrone.
- **Erreurs applicatives stables.** Traduire ORM/framework/tiers vers des codes métier i18n ;
  pour l'inattendu, réponse générique et détail technique dans l'observabilité.
- **Nommage explicite.** Préférer les noms métier complets, sans abréviations locales ; seuls les
  acronymes techniques standard et non ambigus sont acceptables. `process`, `handle`, `manage`
  ou `execute` sont des signaux à remplacer par un verbe métier plus précis quand il existe.
  Extraire une fonction même non réutilisée si son nom clarifie une étape.
- **Standard mature à bénéfice comparable.** Favoriser les technologies largement adoptées pour
  onboarding, debug, sécurité et maintenance. Une technologie niche reste acceptable si son gain
  est concret et qu'elle reste confinée derrière une API simple.
- **Petit helper > petite dépendance.** Le biais naturel est d'installer une librairie ; le
  challenger. Si le besoin tient dans environ 30–50 lignes sans cas limites sérieux, préférer un
  helper interne. Un `common`/`shared` peut servir d'incubateur, puis extraire les groupes cohésifs
  vers des packages spécialisés quand ils émergent.
- **Monolithe modulaire par défaut.** Pour démarrer ou avec une équipe unique, préférer un
  monolithe aux frontières strictes. Les microservices doivent répondre à un besoin réel
  d'autonomie organisationnelle ou de déploiement. Lecture cross-module possible via contrat
  explicite ; écriture uniquement via le module propriétaire.
- **Service avant pureté.** Sur incident, restaurer le service rapidement et tracer la cause
  racine ; ne pas pérenniser un workaround d'un défaut possédé. Refactorer le chemin directement
  concerné si cela simplifie la feature ; demander avant d'élargir aux abstractions voisines.
- **Exploitation intégrée.** Logs structurés et contextualisés sur les chemins critiques ; éviter
  `console.log` et le bruit. Préférer migrations, handlers, workers, backoffice ou CLI supportée
  aux scripts ad hoc ; un script one-shot reste un dernier recours. Supprimer le code mort : Git
  conserve l'historique.
- **Compatibilité selon le contrat de déploiement.** Si producteurs et consommateurs sont
  déployés ensemble, refactorer vite tous les consommateurs. Maintenir une compatibilité stricte
  pour applications mobiles, clients externes ou versions indépendantes.

## Mode de travail

- **Lecture autonome, écriture contrôlée.** Explorer, rechercher et vérifier librement.
  Une tâche évidente peut être implémentée directement ; sinon, demander validation avant
  toute écriture persistante dès que périmètre, architecture ou hypothèse sont incertains.
- **Validation concise.** Donner le périmètre, les changements principaux, hypothèses,
  incertitudes et une recommandation. Pour un changement complexe, préférer une vue du
  diff conceptuel : arborescence, schéma ASCII, maquette CLI/UI ou équivalent.
- **Plan seulement quand il aide.** Sur une tâche simple et stable, plan court si utile.
  Sur une tâche complexe, explorer sans figer tôt la solution et proposer des issues pour
  les blocages découverts.
- **Rester focalisé.** Prioriser blocages, réduction d'incertitude, fonctionnel, puis dette.
  Une dette non bloquante ne mérite une proposition d'issue que si elle touche directement
  la tâche ; ne jamais l'implémenter sans décision explicite.
- **Continuer ce qui est indépendant.** Un blocage local n'arrête pas un travail sans
  dépendance de code ni dépendance architecturale avec lui.
- **Relais sans ancrage.** Pour un nouvel agent, transmettre faits, contraintes et pistes
  invalidées ; ne transmettre des sources précises que si leur pertinence est certaine,
  et ne pas pré-écrire la solution.
- **Contexte comme ressource.** Si la session devient trop chargée pour un nouveau correctif,
  préférer une issue et un prompt concis pour une nouvelle session. ~60 % de contexte visible
  est un signal d'alerte, pas une limite absolue.
- **Barre de vérification.** Lint, types, tests et CI verts restent la barre par défaut, mais ne
  constituent pas une revue. Avant de demander une revue sur une PR que vous ouvrez, passez
  `merge-verdict` sur votre propre PR et corrigez ses constats bloquants avant de solliciter quiconque.

## Review

- Garder les retours courts, hiérarchisés et centrés sur la PR.
- **Style non bloquant.** Signaler la sur-abstraction sans en faire un motif de blocage ; la
  décision reste à l'auteur.
- Un constat important peut devenir une issue. S'il est simple et déjà compris, le reviewer
  peut le corriger immédiatement ; sinon, proposer une issue et un prompt pour un agent frais.
- Une migration de framework trop mécanique est suspecte : repartir des invariants et garanties
  attendues avant de transposer l'ancienne implémentation ou ses paradigmes.

## Points de vigilance

Signaler systématiquement les cas limites et conséquences non voulues du changement :

1. **Gestion d'erreur incomplète** — échecs, timeouts, retours vides/nuls. Sur une écriture
   externe : idempotence nommée, ordre attendu, écriture atomique des valeurs dérivées ; une
   entrée inconnue reste brute, quarantinée et rejouable.
2. **Nommage et lisibilité** — proposer directement de meilleurs noms. Une forte densité de
   commentaires sur le chemin de la tâche est un signal de conception ; proposer une issue,
   sans élargir automatiquement le travail.
3. **Dépendance de trop** — si quelques dizaines de lignes sans cas limites sérieux suffisent,
   proposer d'abord cette option. Pour l'orchestration de promesses, vérifier si les primitives
   natives suffisent avant `p-limit`, `p-map`, `p-queue` ou équivalent.

## Expériences en cours

- Sur le code existant, ~500 lignes modifiées/supprimées = alerte de dérive ; ~800 = forte
  réévaluation : justifier la suite ou proposer un découpage. Exclure code généré, snapshots,
  lockfiles, fixtures et nouveaux tests ; compter les modifications de tests existants.
- Avant une modification non triviale, annoncer les zones prévues et signaler une expansion
  matérielle du périmètre.
