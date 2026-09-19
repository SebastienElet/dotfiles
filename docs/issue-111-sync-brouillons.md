# #111 — Décisions Doctor et brouillons de synchronisation

**D1 et D2 approuvées le 19 septembre 2026.** Le mainteneur a répondu « ok »
aux deux recommandations présentées dans cette session : lecture seule stricte et
conservation des défauts MCP, avec équivalence à portée explicite identique.
**Les huit brouillons S1–S8 du 16 septembre restent non approuvés et non publiés.**
Cible : `SebastienElet/dotfiles`, parent prévu [#111](https://github.com/SebastienElet/dotfiles/issues/111).
Le [bilan sourcé](issue-111-doctor-bilan.md) distingue les décisions, les capacités
livrées et les preuves restant à établir. Cette validation ne vaut ni livraison
de la lecture seule stricte ni autorisation de publier les sous-issues.
Les titres et corps ci-dessous décrivent des résultats attendus ; ils ne prescrivent
ni nouvelle commande, ni modules, ni étapes d'implémentation.

Recherche de doublons ouverte/fermée effectuée avec les termes Arnes, sync,
synchronisation, synchroniser, synchronization, resolver, résolveur, read-only et
lecture seule. Aucun enfant de synchronisation distinct n'a été trouvé. Travaux voisins :
[#166](https://github.com/SebastienElet/dotfiles/issues/166) livré pour l'inventaire Codex,
[#158](https://github.com/SebastienElet/dotfiles/issues/158) ouvert pour les prompts liés,
[#152](https://github.com/SebastienElet/dotfiles/issues/152) ouvert pour une éventuelle
reconstruction explicite. Ce relevé n'empêche pas un doublon créé ultérieurement ;
la recherche sera rafraîchie avant publication.

## Disposition par ressource

| Ressource    | Proposition                        | Limite / dépendance                                                                                                                     |
| ------------ | ---------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| manifest     | Pas de nouvel enfant sync          | Source canonique versionnée, fournie par Moon ; pas de génération depuis l'état observé                                                 |
| config       | S1                                 | Valeurs user déclarées seulement, pas la totalité du fichier natif                                                                      |
| instructions | S2                                 | Claude user/project, Codex user ; représentations déjà diagnostiquées                                                                   |
| skills       | S3                                 | Skills gérés seulement ; inventaires système/plugins hors propriété ; D1 ne doit pas devenir une permission implicite d'effets externes |
| prompts      | S4                                 | File/rendered ; symlink exclu tant que #158 n'est pas disposée                                                                          |
| commands     | S5                                 | Claude user/project ; cohérence avec les prompts, sans double écriture contradictoire                                                   |
| rules        | S6                                 | Claude/Cursor user ; destination divergente conservée                                                                                   |
| hooks        | Pas de nouveau chantier équivalent | `setup hooks --agent …` réconcilie déjà user ; voir état livré ci-dessous                                                               |
| mcp          | S7                                 | Agents/portées explicitement sélectionnés ; D2 approuvée, brouillon S7 toujours à valider avant publication                             |
| statusline   | S8                                 | Codex user/project seulement ; liste ordonnée, pas rendu TUI                                                                            |

Ces périmètres stabilisent uniquement les ressources et représentations déjà observables
par Doctor. Ils ne signifient pas qu'une politique d'écriture a déjà été approuvée.
Chaque brouillon S1–S8 inclut les contraintes communes suivantes dans son corps lors de
la publication ; elles font partie de la présente demande de validation.

## Contraintes communes aux huit brouillons

- La sélection d'agent, de portée et de ressource doit être explicite avant toute mutation.
  Une combinaison non prise en charge ne doit provoquer aucune écriture ; une sélection
  vide doit être distinguée d'une synchronisation réussie.
- Seules les déclarations gérées et leurs projections sont concernées. Sources canoniques,
  ressources non déclarées, secrets, plugins et skills système restent préservés.
  Aucune adoption, suppression générale ou installation/mise à jour d'agent ou de plugin.
- Respecter les ADR-001/003/038 : Moon reste l'orchestrateur, les destinations divergentes
  d'un déploiement ne sont pas écrasées, et aucun lien ne doit permettre d'écrire dans
  une source canonique ou hors de la propriété autorisée. Toute incompatibilité constatée
  exige une décision explicite avant implémentation.
- L'opération annonce son périmètre et distingue ce qu'elle a appliqué, déjà trouvé conforme,
  refusé ou échoué. Une erreur ou un résultat partiel ne doit pas être présenté comme succès.
  Les valeurs sensibles ne doivent pas apparaître dans ses diagnostics.
- Manifeste/source invalide, conflit de propriété, type inattendu ou erreur de lecture
  doivent laisser les destinations concernées intactes. Un échec d'écriture ne laisse
  pas d'artefact tronqué. Le rejeu doit être sûr et, à état conforme, ne rien réécrire.
  Les critères ne promettent pas une transaction globale entre ressources.
- La validation utilise le Doctor existant sur les ressources sélectionnées et des preuves
  de préservation/rejeu/échec à la frontière réelle de mutation. Elle nomme macOS et Ubuntu
  comme exercés ou non vérifiés et distingue fichiers conformes de sessions d'agents.
  Aucun oracle miroir des déclarations ni installation complète du poste n'est demandé.

## D1 — Décider la frontière de lecture seule de Doctor face au résolveur Codex

**Résultat attendu.** Mettre l'inventaire Codex livré par #166/#168 en conformité
avec la lecture seule stricte conservée pour Doctor par la décision du 19 septembre.

**Constat.** Sur `272e8a6`, `doctor skills` et l'agrégat Codex user exécutent les
deux commandes d'inventaire de Codex. L'[expérience macOS](issue-111-doctor-bilan.md#contre-exemple-de-lecture-seule)
obtient code 0 et snapshots identiques malgré les écritures d'un double hors des
arbres observés. Elle ne prouve aucun effet du vrai Codex. HOME/cwd, délai et taille
de sortie bornés ne constituent pas un confinement des effets.

**Décision approuvée.** Doctor doit rester sans écriture ni contact réseau, y compris
par ses subprocessus. La portée comprend les écritures persistantes et transitoires,
dans et hors du HOME et du projet. Une sélection active non observable dans cette
frontière est signalée explicitement comme indisponible, jamais déduite d'un cache
solitaire. L'exhaustivité de l'inventaire Codex peut donc être réduite ; l'exigence
de sélection autoritative de #166 est conservée pour les résultats observables.

Le correctif de la PR #343 supprime maintenant le résolveur ; le contre-exemple
historique est devenu une régression rouge puis verte. Le bilan distingue les
preuves locales du correctif, la CI du head final et son intégration encore attendue.

**Disposition et preuves restantes.**

- [x] Le mainteneur conserve la lecture seule stricte pour les effets directs et ceux
      des processus enfants ; l'indisponibilité de l'inventaire doit être explicite.
- [x] La sélection autoritative de #166 est conservée ; une couverture moindre est
      acceptée lorsque cette sélection n'est pas observable dans la frontière retenue.
- [ ] Le contrat de #111/#123/#124 est réconcilié sans qualifier leurs anciens snapshots
      de preuve universelle ; #126 demeure la référence unique.
- [ ] Les preuves attendues pour l'implémentation choisie nomment plateformes, limites et
      frontière exercée, et comprennent le contre-exemple de subprocessus à effets.
- [ ] Aucun passage d'agent réel n'est revendiqué sans observation et version nommées.

**Hors périmètre.** Synchroniser les plugins, choisir un mécanisme technique à la place
de l'implémenteur, ajouter une nouvelle infrastructure de validation dans l'issue de décision.
Parent proposé : #111 ; lien de contexte : #166, sans rouvrir son implémentation par défaut.

## D2 — Préciser l'équivalence MCP entre Doctor direct et agrégé

**Résultat retenu.** Le critère « mêmes diagnostics » de #111/#123 s'applique à
une portée explicitement identique ; les défauts des deux appels restent inchangés.

**Constat.** Le [contre-exemple reproductible](issue-111-doctor-bilan.md#contre-exemple-déquivalence-sans-scope-mcp)
donne 0/unsupported user en direct et 1/drift project dans l'agrégat. Le comportement
est livré, testé et déjà documenté par #126 ; un scope explicite produit les mêmes
diagnostics MCP. Le changer modifierait ce comportement public.

**Décision approuvée.** `doctor mcp` conserve `user` par défaut ; l'agrégat sans
`--scope` examine toutes les portées MCP déclarées. L'équivalence des diagnostics MCP
est garantie dans les représentations vérifiées lorsque `--scope` est explicite et
identique, à filtre agent identique. Aucune équivalence n'est promise entre les deux
appels sans scope. La décision conserve le comportement déjà livré et documenté.

**Disposition et suivi.**

- [x] #111/#123 expriment sans ambiguïté la sélection à laquelle s'applique l'équivalence.
- [x] Le mainteneur valide les défauts actuels et limite l'équivalence aux sélections
      explicites identiques. Cette disposition a été reportée dans #111/#123 et relue.
- [ ] Le cas MCP uniquement project a un résultat attendu explicite pour les appels
      direct, agrégé et filtré, dans les deux formats, sans perte silencieuse de diagnostic.
- [ ] La référence commune reste exacte ; toute modification du défaut public est
      justifiée et couverte par le cas contradictoire, sans refaire les tests sans rapport.

**Hors périmètre.** Uniformiser tous les défauts des autres ressources ou synchroniser MCP.
La décision est consignée ici ; elle peut être reportée dans #111/#123 lors d'une
mise à jour autorisée, sans créer une issue autonome supplémentaire.

## S1 — Synchroniser les valeurs user de configuration gérées par Arnes

**Résultat attendu.** Appliquer à l'agent sélectionné les valeurs `user_config` du
manifeste, sans remplacer sa configuration native entière. Doctor compare déjà ces
valeurs pour Claude, Cursor et Codex ([#115](https://github.com/SebastienElet/dotfiles/issues/115),
[contrat actuel](arnes-doctor.md#ressources-et-différences-entre-agents)).

**Critères d'acceptation**, avec les contraintes communes :

- [ ] Les valeurs déclarées absentes ou différentes deviennent conformes dans le format
      natif pris en charge ; Doctor config les rapporte healthy après l'opération.
- [ ] Les valeurs non déclarées et les autres sections restent inchangées, y compris
      hooks, MCP et statusline ; les défauts user ne sont jamais appliqués à project.
- [ ] Un fichier malformé, un lien vers une source canonique, une destination conflictuelle
      ou une propriété incertaine sont signalés et préservés, sans réparation implicite.
- [ ] Le rejeu conforme, les refus et un échec d'écriture ont une preuve observable
      adaptée au fichier partagé ; aucune configuration tronquée ni succès partiel masqué.

**Hors périmètre.** Installation de binaires, réglages non déclarés, politique complète
de configuration project et synchronisation des autres ressources.
Parent proposé : #111 ; observation livrée : #115.

## S2 — Synchroniser les projections d'instructions prises en charge

**Résultat attendu.** Rendre conformes les projections déclarées de Claude user/project
et Codex user à partir de leurs sources, en conservant les représentations de
[#117](https://github.com/SebastienElet/dotfiles/issues/117) et de l'ADR-003.

**Critères d'acceptation**, avec les contraintes communes :

- [ ] Les liens Claude user, includes Claude project et contenu assemblé Codex user
      sélectionnés sont conformes au Doctor après l'opération.
- [ ] Une projection absente peut être créée ; un résultat assemblé obsolète est
      actualisé selon l'ADR-003 ; un lien ou fichier divergent est préservé avec refus explicite.
- [ ] Les instructions locales hors projection restent intactes ; une destination
      project préexistante sans propriété démontrée n'est pas réécrite pour ajouter un include.
- [ ] Source manquante, include cyclique ou hors frontière et erreur d'écriture n'altèrent
      pas les instructions concernées ; rejeu conforme sans réécriture.

**Hors périmètre.** Modifier les instructions elles-mêmes, ajouter le support Cursor
ou Codex project, prétendre qu'un agent a effectivement chargé la projection.
Parent proposé : #111 ; observation livrée : #117.

## S3 — Synchroniser les installations de skills gérés

**Résultat attendu.** Converger les projections de skills possédées par le dépôt,
pour les agents et scopes déjà diagnostiqués par [#118](https://github.com/SebastienElet/dotfiles/issues/118).
Les sources user et project restent distinctes selon ADR-028/040.

**Critères d'acceptation**, avec les contraintes communes :

- [ ] Les liens feuille user et racine project absents sont établis selon les déclarations
      et les projections correctes restent inchangées ; Doctor confirme les skills gérés sélectionnés.
- [ ] Les sources restent canoniques ; `SKILL.md` ou ressource relative manquante bloque
      la projection concernée sans copie de remplacement ni installation d'une dépendance.
- [ ] Un lien divergent ou une destination locale est préservé et signalé. Une skill
      retirée du manifeste n'est ni adoptée ni supprimée implicitement.
- [ ] Aucun plugin ou skill système n'est modifié ; l'inventaire externe indisponible
      reste une limite distincte, jamais une preuve de conformité ou une permission d'effet externe.
- [ ] Les preuves couvrent sélection, collisions, liens cassés et rejeu, sans confondre
      conformité gérée avec activation en session. D1 reste applicable aux appels Doctor Codex.

**Hors périmètre.** Gestion des plugins, validation qualitative de `skill-manager`,
réécriture de skills, élagage et génération du manifeste depuis l'installation.
Parent proposé : #111 ; observation livrée : #118 ; frontière externe : D1.

## S4 — Synchroniser les projections file et rendered de prompts

**Résultat attendu.** Produire les projections déclarées de prompts réutilisables
prises en charge par [#109](https://github.com/SebastienElet/dotfiles/issues/109) :
Claude user/project et Cursor project, selon leur représentation reconnue par Doctor.

**Critères d'acceptation**, avec les contraintes communes :

- [ ] Une projection absente ou un contenu rendu dont la propriété est établie devient
      conforme à la source, aux includes et aux variables déclarées ; Doctor prompts le confirme.
- [ ] Source illisible, include non résolu/cyclique, variable non déclarée ou collision
      empêchent de modifier la destination concernée et produisent un refus explicite.
- [ ] Les prompts locaux ou possédés par plugins restent intacts ; un fichier divergent
      sans propriété établie n'est pas réécrit pour le faire correspondre au manifeste.
- [ ] La synchronisation du contenu préserve la cohérence des liaisons de commandes
      déclarées et ne crée pas deux versions concurrentes du même artefact ; rejeu sans réécriture.

**Hors périmètre.** Invocation du prompt, création de noms de commandes, Codex,
Cursor user et projections symlink : cette dernière décision reste dans #158.
Parent proposé : #111 ; observation livrée : #109 ; cohérence avec S5.

## S5 — Synchroniser les liaisons de commandes Claude déclarées

**Résultat attendu.** Exposer les liaisons user/project déclarées entre commandes
Claude et prompts, avec leur nom et description attendus, selon
[#110](https://github.com/SebastienElet/dotfiles/issues/110).

**Critères d'acceptation**, avec les contraintes communes :

- [ ] Une liaison sélectionnée et sa projection reconnue deviennent conformes au Doctor
      commands ; leur contenu reste cohérent avec le prompt canonique et le Doctor prompts.
- [ ] Deux déclarations incompatibles visant le même artefact ou un nom déjà possédé
      par une ressource locale/plugin sont refusées sans écrasement ni adoption.
- [ ] Une source de prompt invalide ou un échec d'écriture ne laisse pas de commande
      partiellement rendue ; nom, description et contenu restent cohérents après reprise.
- [ ] Les passages prompts puis commands, ou commands puis prompts, n'introduisent
      aucune divergence lorsque leurs déclarations sont compatibles ; rejeu conforme sans mutation.

**Hors périmètre.** Exécuter les commandes, inventer un support Cursor/Codex ou symlink,
renommer des commandes non gérées et dupliquer l'autorité sur le contenu des prompts.
Parent proposé : #111 ; observation livrée : #110 ; cohérence avec S4.

## S6 — Synchroniser les liens de rules user déclarées

**Résultat attendu.** Établir les projections de rules Claude et Cursor user déjà
observées par [#119](https://github.com/SebastienElet/dotfiles/issues/119), sans en modifier le contenu.

**Critères d'acceptation**, avec les contraintes communes :

- [ ] Une destination absente reçoit la projection déclarée ; Doctor rules la confirme.
      Un lien correct reste inchangé et le rejeu ne produit aucune réécriture.
- [ ] Un fichier local, un lien vers une autre cible ou une collision est préservé
      et signalé ; aucune reconstruction implicite de destination divergente.
- [ ] Source absente, mauvais type ou erreur d'accès ne crée pas de projection cassée.
- [ ] Sélectionner un agent/une portée ne modifie aucune rule voisine ; les scénarios
      de refus et d'échec sont vérifiés sur les plateformes explicitement nommées.

**Hors périmètre.** Rules project, Codex, contenu des règles, application réelle par
l'agent et reconstruction globale suivie par #152.
Parent proposé : #111 ; observation livrée : #119.

## S7 — Synchroniser les enregistrements MCP gérés sans démarrer de service

**Résultat attendu.** Converger les enregistrements locaux déclarés pour un agent
et une portée explicitement sélectionnés, selon [#121](https://github.com/SebastienElet/dotfiles/issues/121).
D2 est désormais tranchée ; la validation de S7 et l'autorisation de sa publication
restent nécessaires, comme pour les autres brouillons de synchronisation.

**Critères d'acceptation**, avec les contraintes communes :

- [ ] Commande, arguments ordonnés, références d'environnement et état enabled lorsqu'il
      est pris en charge correspondent aux déclarations ; Doctor MCP sélectionné le confirme.
- [ ] Les enregistrements non gérés et autres sections du fichier natif restent inchangés.
      Les valeurs de secrets ne sont ni copiées dans le manifeste ni exposées dans les sorties.
- [ ] Une collision avec l'autre portée est diagnostiquée sans déplacer ni supprimer
      l'enregistrement concurrent ; des champs non interprétables sont préservés avec refus.
- [ ] Aucune commande de serveur, conteneur ou service distant n'est lancée ou sondée.
      Une commande requise indisponible laisse l'état concerné intact avec résultat explicite.
- [ ] Les erreurs de lecture/écriture et le rejeu n'endommagent pas le fichier partagé ;
      les limites par agent, notamment celles d'enabled, restent explicites.

**Hors périmètre.** Installer MCP, gérer Docker, authentifier ou tester les services,
adopter les enregistrements de plugins et supprimer les collisions automatiquement.
Parent proposé : #111 ; observation livrée : #121 ; décision de portée approuvée : D2.

## S8 — Synchroniser la liste de statusline Codex déclarée

**Résultat attendu.** Converger la liste ordonnée de statusline Codex user ou project
à la déclaration reconnue par [#122](https://github.com/SebastienElet/dotfiles/issues/122).

**Critères d'acceptation**, avec les contraintes communes :

- [ ] Une liste absente ou différente dans la portée explicitement sélectionnée devient
      conforme, avec l'ordre exact déclaré ; Doctor statusline sélectionné la confirme.
- [ ] Les autres clés TOML, y compris celles de TUI, MCP et de configuration du modèle,
      restent inchangées ; l'autre portée n'est pas modifiée.
- [ ] Configuration invalide, lien hors propriété ou erreur d'écriture laissent la
      configuration concernée intacte ; rejeu conforme sans réécriture.
- [ ] Une absence de déclaration ou un agent non pris en charge provoque un résultat
      explicite sans mutation ; un rapport Doctor vide ne constitue pas le succès d'une synchronisation.

**Hors périmètre.** Statuslines Claude/Cursor, lancement de commande, rendu TUI réel
ou élargissement des items reconnus par Doctor.
Parent proposé : #111 ; observation livrée : #122.

## Hooks : partir de la réconciliation déjà livrée

[Setup](../tooling/arnes/src/hooks.rs) exige un agent, prend user par défaut et
refuse project. Il réconcilie les hooks possédés et peut retirer les entrées
possédées qui ne sont plus déclarées ; ce comportement existant n'est pas une
autorisation d'élagage pour les huit autres propositions.
Les suites [hooks_setup](../tooling/arnes/tests/hooks_setup.rs),
[hooks_reconciliation](../tooling/arnes/tests/hooks_reconciliation/) et
[hooks_doctor](../tooling/arnes/tests/hooks_doctor.rs) couvrent déjà cette frontière.
Les [tâches Claude](../.moon/tasks/harness-claude.yml) et
[Codex](../.moon/tasks/harness-codex.yml) utilisent setup avec Doctor comme contrôle.
Le support de setup Cursor n'atteste pas une intégration Moon Cursor équivalente.

Aucun besoin résiduel de synchronisation hooks n'a été démontré : créer une nouvelle
issue d'implémentation équivalente dupliquerait la livraison. Proposition : conserver
#120 comme référence de cette ressource et ne créer un enfant additionnel que pour
un écart concret approuvé. Ni façade générale ni extension project ne sont demandées.

## Cohérence et publication

Chaque promesse des titres S1–S8 est couverte par un résultat Doctor sélectionné,
la protection des voisins et des refus/échecs observables. Les limites d'agents et
de représentations empêchent une promesse de parité ; les contraintes communes font
partie de chaque corps. D1 et D2 sont approuvées ; le correctif D1 est livré dans
la PR, tandis que D2 conserve le comportement existant. Aucun brouillon S1–S8 n'est
approuvé par la seule résolution de ces arbitrages.

Après validation des corps : rafraîchir la recherche de doublons, publier les seuls
brouillons approuvés, inclure leurs contraintes communes et vérifier leur rattachement
réel à #111. D2 est reportée dans les trackers ; S7 n'attend plus un choix de portée.
Aucune publication, modification d'issue, fermeture ou synchronisation n'a lieu dans ce dossier.
