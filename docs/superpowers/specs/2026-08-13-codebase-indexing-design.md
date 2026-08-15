# Indexation des bases de code pour les agents — Design

**Date** : 2026-08-13 · **Statut** : approuvé

## Objectif

Traiter l'issue #86 en choisissant sur mesures une technologie de récupération adaptée aux grands
dépôts, puis en prouvant son usage par Claude Code, Codex et Cursor. CodeGraph reste un candidat :
la décision peut retenir Serena, Zoekt ou conclure qu'aucune solution ne dépasse suffisamment
`rg`.

Un grand dépôt de référence, non identifié dans ce dépôt public et figé à un commit avant toute
mesure, constitue le corpus principal. Le présent dépôt constitue le contrôle négatif : son volume
est proche de 200 000 tokens et aucun agent ne doit y déclencher l'index. Les sources du corpus ne
quittent pas la machine ; la télémétrie, les vérifications de mise à jour et les fournisseurs
distants sont désactivés.

## Approches considérées

1. Séparer le benchmark natif de la mesure d'adoption, puis intégrer le gagnant par MCP. Chaque
   moteur est d'abord évalué avec sa propre interface ; le comportement des trois agents est mesuré
   seulement après la sélection. **Approche retenue.**
2. Placer une façade MCP uniforme devant tous les candidats. L'instrumentation serait homogène,
   mais la façade réduirait les moteurs à leur dénominateur commun et sa qualité deviendrait une
   variable du benchmark.
3. Tester immédiatement chaque couple moteur-agent. Cette approche refléterait l'usage final, mais
   confondrait la qualité de récupération, la capacité du modèle à découvrir les outils et les
   différences de configuration des clients.

La séparation retenue n'interdit pas un lanceur commun après la décision. Celui-ci ne sert qu'au
diagnostic et au cycle de vie ; il ne réécrit ni ne normalise les requêtes du moteur.

## Candidats

Quatre bras couvrent des familles distinctes :

- `@colbymchenry/codegraph`, graphe structurel persistant local exposé par MCP ;
- `oraios/serena`, navigation symbolique fondée sur les serveurs LSP et exposée par MCP ;
- `sourcegraph/zoekt`, index lexical local à trigrammes, interrogé par son interface native ;
- `rg` et `fd`, contrôle sans index conforme à l'ADR-015.

Les versions, sommes de contrôle, commandes d'installation et options réellement utilisées sont
figées avant le premier run. Aucun installateur fournisseur n'est autorisé à modifier directement
les fichiers de Claude Code, Codex ou Cursor. Si Zoekt gagne, un adaptateur MCP en lecture seule est
admis après le benchmark, car Zoekt n'en fournit pas ; cet adaptateur fait alors partie du prototype
multi-agent et de ses évaluations, pas du benchmark natif.

CodeGraph est identifié par son package npm et son dépôt afin d'éviter les homonymes. Serena est
limitée à ses outils de lecture et de navigation : ses outils d'édition, de shell et ses hooks de
rappel restent désactivés. Zoekt utilise un stockage local et Universal Ctags uniquement si la
version retenue en dépend pour son classement symbolique.

## Corpus et vérité terrain

Le corpus comporte douze questions, réparties également entre quatre catégories :

1. navigation vers le composant responsable d'un comportement ;
2. dépendances directes et transitives entre composants ;
3. définition, références et appels d'un symbole ;
4. impact inter-fichiers et inter-packages d'une modification.

Chaque question possède avant exécution : une formulation stable, l'ensemble attendu de fichiers
et symboles, les facettes obligatoires de la réponse, les faux positifs connus et un budget maximal
de cinq appels de récupération. La vérité terrain est établie directement depuis les sources par
deux passes indépendantes, puis figée avec le commit du corpus. Les plans de requête natifs propres
à chaque candidat sont écrits avant d'observer leurs résultats afin de permettre à chaque famille
d'utiliser ses capacités sans adaptation opportuniste.

Les sorties brutes susceptibles de contenir du code, des chemins ou des noms propres restent dans
un répertoire local ignoré. Le dépôt versionne le corpus expurgé, les facettes de notation, les
commandes, l'environnement, les agrégats et les empreintes des traces. Un contributeur autorisé à
lire le corpus peut régénérer les traces complètes ; le dépôt public ne publie aucun extrait ni
identifiant du corpus.

## Méthode et score

Chaque mesure native est répétée trois fois dans le même environnement macOS, après remise du
candidat dans l'état défini par le scénario. Les résultats nomment explicitement le matériel, le
système, le shell, les versions et les cibles supportées non exercées. Linux n'est pas déclaré
validé par une exécution macOS.

Le score total est pondéré ainsi :

| Axe | Poids | Mesure |
| --- | ---: | --- |
| Qualité de récupération | 40 | 20 points de pertinence et 20 de complétude, notés en aveugle contre la vérité terrain |
| Fraîcheur et robustesse | 20 | détection et traitement corrects des mutations et des modes de panne |
| Performance et ressources | 15 | indexation initiale, synchronisation, latence p50/p95, CPU, RSS et disque |
| Intégration et adoption | 15 | MCP, disponibilité des outils et comportement observé sur les trois agents |
| Confidentialité et maintenance | 10 | fonctionnement local, licence, surface d'installation, mise à jour et suppression |

La pertinence mesure la proportion des résultats utiles dans le budget ; la complétude mesure la
proportion des fichiers, symboles et facettes attendus effectivement retrouvés. Les résultats sont
notés par facette sans connaître le candidat. Les autres axes utilisent une grille publique de 0 à
5 par sous-critère, convertie linéairement vers leur poids.

Un candidat n'est sélectionnable qu'à partir de 75/100 et si aucun critère éliminatoire n'échoue :

- fonctionnement local sans sortie du code ni télémétrie ;
- détection d'un index absent, périmé ou inutilisable sans réponse silencieusement obsolète ;
- couverture suffisante des langages représentant au moins 90 % des fichiers source éligibles ;
- interface MCP maintenable pour Claude Code, Codex et Cursor ;
- absence de corruption persistante après le cycle de suppression et reconstruction documenté.

Le meilleur score parmi les candidats admissibles est retenu. Si aucun candidat ne passe les
barrières, l'ADR conclut qu'aucune indexation n'est imposée et conserve `rg` comme solution.

## Mesure du seuil d'activation

Le seuil inférieur de 200 000 tokens est emprunté à la mesure publiée d'Anthropic : sous ce volume,
le corpus entier peut entrer dans la fenêtre et le présent dépôt doit rester sans index. Le seuil
supérieur est mesuré à partir de paliers du corpus dont la composition par langage et package est
conservée.

Le seuil retenu est le premier palier supérieur à 200 000 tokens où le candidat améliore, sur deux
paliers successifs, soit la qualité d'au moins 10 points, soit la latence d'au moins 25 %, sans
régression de qualité. En l'absence de deux paliers satisfaisants, aucune obligation d'indexation
n'est créée.

## Diagnostic partagé

Un outil local `code-index doctor` décide si le dépôt courant doit être indexé sans dépendre du
client agent. Il ne sert pas les requêtes et n'intercepte aucune commande.

Le diagnostic :

1. inventorie les fichiers retournés par `git ls-files` ;
2. exclut les fichiers ignorés, binaires, générés, vendored, sensibles ou porteurs de secrets selon
   la configuration partagée ;
3. calcule le volume éligible et une estimation conservative en tokens ;
4. compare la distribution des langages à la matrice de couverture du moteur retenu ;
5. produit exactement un verdict : `below-threshold`, `index-required` ou `unsupported` ;
6. compare l'ancre de fraîcheur enregistrée à `HEAD`, à l'index Git et aux fichiers suivis modifiés.

`unsupported` nomme les langages ou exclusions responsables et impose le mode dégradé. Le
diagnostic retourne un statut non nul seulement lorsqu'un agent s'apprête à utiliser un index
requis mais absent, périmé ou non couvert ; la simple absence d'index sous le seuil est saine.

## Cycle de vie et modes de panne

Le cycle commun est : diagnostic, initialisation ou synchronisation, requête du moteur, validation
des conclusions critiques dans les fichiers source, puis contrôle de `git status`. L'ancre de
fraîcheur est mise à jour uniquement après une synchronisation réussie.

Le benchmark couvre :

- index absent ;
- modification d'un fichier suivi ;
- renommage et suppression ;
- changement de branche ;
- arrêt du serveur ;
- index volontairement corrompu ;
- langage non couvert, avec une fixture locale expurgée comme cas négatif.

Une indisponibilité autorise un diagnostic et une seule tentative de synchronisation ou de
reconstruction. En cas de nouvel échec, l'agent annonce le mode dégradé et limite `rg`/`fd` aux
sous-arbres identifiés ; il ne remplace pas l'index par une boucle récursive silencieuse sur tout le
dépôt.

Les index, journaux complets et états machine résident hors du dépôt sous un emplacement de cache
local. La mesure détermine le quota documenté du moteur retenu ; la commande de suppression efface
un index de dépôt identifié sans toucher aux autres caches.

## Intégration multi-agent

Le moteur retenu est exposé par MCP. Claude Code, Codex et Cursor utilisent le même exécutable de
lancement et le même cycle de vie, avec seulement les adaptateurs de configuration exigés par leur
format. Codex utilise la configuration MCP de projet supportée par le CLI et l'extension IDE ; les
configurations Claude Code et Cursor suivent leurs mécanismes natifs équivalents.

Le `Makefile` installe les dépendances et distribue les adaptateurs sans exécuter les assistants de
configuration des fournisseurs. La configuration versionnée désactive la télémétrie et les appels
de mise à jour. Un serveur HTTP partagé n'est pas imposé : le transport stdio est le dénominateur
commun, évite un registre de ports et conserve une instance par dépôt ou worktree.

La skill canonique vit sous `.agents/skills/code-indexing/` et est distribuée aux trois agents selon
l'ADR-028. Elle contient la matrice de routage :

- index requis pour exploration ouverte au-dessus du seuil, architecture, références/appels,
  impact et recherche inter-packages ;
- `rg`/`fd` préférés pour chemin connu, littéral exact, expression régulière et vérification ciblée ;
- diagnostic et fraîcheur avant toute requête d'index ;
- validation des résultats importants contre la source ;
- mode dégradé explicite et borné.

La règle courte admise dans `ai/AGENTS.md` reste agent-agnostique conformément à l'ADR-025. Son
texte exact n'est ajouté que si l'ablation marginale exigée par l'ADR-036 démontre un effet. Aucun
hook n'intercepte ou ne réécrit les recherches : l'ADR-033 reste en vigueur et l'intégration complète
l'ADR-015.

## Évaluations d'adoption

Les évaluations comportementales portent uniquement sur le candidat retenu et sur le texte exact de
la skill et de la règle proposées. Chaque agent exécute trois réplicats des scénarios suivants :

- positif : exploration ouverte du grand corpus, avec usage de l'index avant toute recherche large ;
- négatif : recherche exacte dans le présent petit dépôt, sans invocation de l'index ;
- fraîcheur : modification contrôlée, détectée puis synchronisée avant réponse ;
- ablation : scénario positif identique sans la skill ni la règle candidates ;
- placebo requis par l'ADR-036 pour isoler l'effet du contenu d'instructions.

Les traces enregistrent les invocations d'index, leur latence, la fraîcheur, les appels shell, le
nombre de fichiers et le volume de sortie recherchés. La règle n'est admise que si elle augmente
l'usage correct de l'index sur le grand dépôt sans déclencher l'index sur le contrôle négatif, sans
masquer un état périmé et sans dégrader la qualité de récupération.

Claude Code et Codex sont obligatoires. Cursor est également exercé ; une impossibilité de lancer le
même MCP ou de charger la skill est une limitation bloquante, pas une intégration supposée.

## Livrables et découpage

L'implémentation produit des commits révocables :

1. harness, corpus expurgé, grille et scripts de collecte ;
2. exécution native et résultats des quatre bras ;
3. prototype du candidat retenu et diagnostic partagé ;
4. évaluations multi-agent et ablation de gouvernance ;
5. skill, règle admise, distribution et configuration finale ;
6. ADR-038, rapport final et plan d'installation, synchronisation, mise à jour et suppression.

L'ADR-038 enregistre la décision, y compris une conclusion négative, le seuil mesuré et la relation
avec les ADR-015, ADR-025, ADR-028, ADR-033 et ADR-036. La fermeture de #86 intervient seulement
après vérification des livrables intégrés, jamais sur le seul design ou sur des mesures locales non
reproductibles.

## Vérification du changement

La vérification du dépôt couvre les extensions réellement modifiées. Les scripts shell passent leur
analyse syntaxique et leurs tests dédiés ; les cibles d'installation sont inspectées puis exercées
uniquement avec `make -n` tant qu'elles peuvent muter Homebrew, Volta ou un autre état global. La
skill passe `skill-manager doctor`, la resynchronisation déterministe de l'index et les évaluations
d'activation.

Le rapport final distingue les preuves produites sur macOS des plateformes non exercées, publie les
commandes exactes et vérifie `git diff --check` ainsi que `git status`. Il ne qualifie aucune barrière
de verte si elle ne couvre pas effectivement les fichiers touchés.
