# #188 — Sous-issues proposées, non publiées

Brouillons du 22 septembre 2026, fondés sur l'[inventaire](issue-188-shell-inventory.md).
Recherche GitHub ouverte/fermée, douze enfants #188 et PR adjacentes consultés : #200 couvre les
tests de déploiement, #299 l'orchestration, #346 l'installation initiale remem. Aucun enfant trouvé
pour les deux résultats spécifiques ci-dessous. Revérifier les doublons au moment de publier.
Le retrait du contrôle textuel PR feedback se coordonne avec #160/#159 ; aucun troisième ticket
de règles du harnais n'est proposé.

## A — Sortir les mutations structurées de l'installation remem du Shell

### Résultat attendu

Les installations remem Codex/Claude et du worker restent accessibles par les tâches Moon
publiques actuelles. Leurs décisions sur la configuration et l'état du service quittent Shell,
sans réimplémenter le stockage, le chiffrement, le modèle ou le worker upstream.

### Constat et périmètre proposé

Sur `3767a7a`, `.moon/tasks/harness-remem.yml` modifie TOML, JSON et plist et décide du remplacement
de l'enregistrement launchd. ADR-041 accepté réserve ces responsabilités à un utilitaire typé.
#346 a exécuté les points d’entrée et leur rejeu sans erreur sur macOS ; le smoke ne capture pas
tous ces fichiers et il manque
des scénarios d'échec isolés. Ce constat n'affirme pas une corruption actuelle.

Conserver noms des tâches, destinations, scope MCP user, désactivation des mémoires natives,
autorisations ciblées Codex, environnement du serveur, label `dev.remem.worker`, intervalle de
300 secondes et mode `worker --once`. Utiliser les interfaces natives supportées lorsqu'elles
possèdent déjà l'opération ; vérifier leur contrat actuel avant de choisir l'implémentation.
La frontière proposée est Moon + petit utilitaire TypeScript ; le runtime remem reste externe.

### Acceptation proposée, à valider

Ces critères ne sont pas encore approuvés. ADR-041 fixe la frontière de langage ; il ne suffit
pas à approuver les garanties détaillées de conservation, publication et reprise ci-dessous.

- [ ] Les points d'entrée Moon réellement livrés sont exercés sur HOME et service isolés ;
      installation initiale et rejeu conservent la configuration étrangère, les champs possédés,
      permissions requises et le contrat de sortie.
- [ ] Entrées invalides/illisibles, CLI absente/échouée, publication impossible et échecs
      launchctl ont des codes non nuls et diagnostics actionnables ; l'état après effet partiel
      est explicite et le rejeu ne perd pas une configuration étrangère.
- [ ] Les documents structurés sont validés à la frontière ; la politique n'est ni copiée dans
      un test ni déplacée dans un nouveau bloc Shell embarqué.
- [ ] L'oracle natif du service est exercé sur macOS jetable ; les effets qui ne peuvent pas être
      atomiques sont nommés. Aucun succès MCP/worker n'est déduit d'une simple présence de fichier.
- [ ] `bun test`, TypeScript 7 épinglé `tsc --noEmit`, contrôles existants pertinents et smoke macOS
      passent ; plateformes et substitutions de processus sont nommées.

### Conséquences et dépendances

Nouveaux modules/tests bornés possibles, dépendants de la toolchain partagée ; aucun framework
de configuration générique. #346 est la provenance, #188 le parent proposé. #160 exclut toute
modification des instructions permanentes ; les skill/règles mémoire et Cursor restent inchangés.
L'installation remem actuelle est macOS ; aucune promesse de service Linux ajoutée.

## B — Unifier le refus des collisions des liens optionnels

### Résultat attendu

Les installations optionnelles PostgreSQL, Cursor et du lanceur Scrapling utilisent la même
politique possédée de collision que les déploiements déjà migrés. Une destination absente est
créée, un lien correct reste silencieux, une destination étrangère est conservée et refusée.

### Constat et périmètre proposé

Sur `3767a7a`, `Makefile:CREATE_SYMLINK` classe et compare les destinations, alors que
`tooling/deploy-link.ts` possède déjà ce comportement testé. Les consommateurs restants sont
`~/.psqlrc`, les liens de skills/règle Cursor et `${LOCAL_BIN}/scrapling_mcp`.
La réduction de duplication justifie la migration ; zéro Make/Shell n'est pas le résultat attendu.

Préserver interfaces appelables, destinations, sources, dépendances et refus des collisions.
Réutiliser le mécanisme de déploiement existant ; aucun helper générique ni réparation silencieuse.
Coordonner les recettes avec #299 ; le graphe optionnel n'est pas redessiné dans cette sous-issue.

### Acceptation

- [ ] Chaque famille de consommateurs est exercée par son vrai point d'entrée conservé, sans
      installation de paquet globale dans une fixture HOME.
- [ ] Absence, lien correct, fichier/répertoire divergent, lien cassé/divergent, parent occupé et
      échec de création donnent les effets/codes requis, sans écrasement ni lien créé à l'intérieur
      d'une destination existante ; rejeu silencieux vérifié.
- [ ] Les destinations et consommateurs sont conservés ; après migration du dernier appel,
      `CREATE_SYMLINK` et ses assertions Shell obsolètes sont retirés, sans copie du helper.
- [ ] Les tests de déploiement existants, `bun test`, `tsc --noEmit` épinglé et contrôles pertinents
      passent. Les liens portables sont exercés sur macOS et Ubuntu ; le poste complet reste macOS.

### Conséquences et dépendances

Les recettes concernées appelleront Bun et devront déclarer sa préparation sans dépendre par
hasard du runner. Examiner les octets des diagnostics actuels avant adaptation ; préserver au
minimum les codes et garanties contractuels, consigner tout écart soumis à validation.
#200 fournit les oracles de déploiement à compléter, #299 le chantier d'orchestration adjacent,
#152 la décision indépendante de reconstruction. Aucun nettoyage général ni changement des
instructions ou du moteur mémoire Cursor inclus.
