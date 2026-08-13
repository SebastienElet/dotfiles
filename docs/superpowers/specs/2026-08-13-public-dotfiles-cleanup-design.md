# Assainissement du dépôt public — Design

**Date** : 2026-08-13 · **Statut** : approuvé

## Objectif

Traiter l'issue #64 en retirant du dépôt public les configurations et automatismes professionnels
devenus inutiles, sans privatiser le dépôt ni réécrire son historique. Le changement conserve les
outils encore utilisés : AWS CLI, Cursor CLI, les adaptateurs `.cursor/` et l'interface Fish `gp`.

L'issue #54 est absorbée par ce chantier, puisque les scripts AWS et le système de hooks de push
qu'elle visait disparaissent entièrement.

## Approches considérées

1. Corriger uniquement les chemins énumérés par #64. Cette approche réduit le diff, mais laisse les
   mêmes noms professionnels dans d'autres exemples et conserve des composants morts liés aux
   éléments retirés.
2. Assainir toutes les occurrences identifiées dans le périmètre et reconstruire `skill-manager`
   depuis les améliorations génériques de `~/Code/brain`, tout en gardant l'architecture des
   dotfiles. **Approche retenue.**
3. Anonymiser tout le dépôt et réécrire l'historique. Aucun secret n'ayant été détecté, cette
   approche détruirait du contexte personnel utile sans réduire un risque établi.

## Suppressions

### Automatismes AWS

Supprimer :

- `scripts/import-instance-start` ;
- `scripts/import-instance-stop` ;
- `scripts/cheque-parser-instance-reboot` ;
- leurs trois aliases dans `fish/conf.d/aliases.fish`.

La cible `aws` du `Makefile` reste en place : AWS CLI pourra encore servir à de futurs usages sans
réintroduire les noms d'instances ni les profils supprimés.

### Configuration de l'IDE Cursor

Supprimer le répertoire versionné `cursor/` et les prérequis du `Makefile` qui installent Cursor.app,
les settings, les extensions et les keybindings de l'IDE. La cible `cursor` devient une cible CLI :
elle conserve `cursor-agent` et le déploiement du skill nécessaire sous `~/.cursor/skills`.

Ne pas toucher à `.cursor/`, qui reste un adaptateur agent utilisé par Cursor CLI. Les symlinks et
skills qu'il expose continuent d'avoir `.agents/skills/` pour source canonique conformément à
l'ADR-028.

### Système de hooks de push

Supprimer :

- `scripts/git_hook_push` ;
- `scripts/git_hook_assert_typos` ;
- `scripts/git_hook_assert_todoes` ;
- `scripts/git_hook_assert_eslint` ;
- `scripts/git_hook_assert_empty_files` ;
- `scripts/git_hook_detect_copy_paste`.

Le système est inactif depuis que `git_hook_assert_todoes` source `git_main_branch`, dont `exit 0`
termine le wrapper au chargement. Il est supprimé plutôt que réparé, car aucun de ses contrôles
n'est encore demandé. `git_main_branch` reste en place pour ses autres appelants actifs.

Remplacer l'alias Fish `gp` par une abbreviation vers `git push`, selon l'idiome consacré par
l'ADR-012. `gpsup` reste inchangé.

Supprimer `docs/adr/020-hooks-de-push-maison.md` et sa ligne d'index : la décision n'est plus en
vigueur et le dépôt ne conserve que les ADR actives. L'ADR-021 reste valide, puisque la détection de
branche principale est encore utilisée ailleurs.

## Données et exemples publics

Vider `dict/user.txt` sans supprimer le fichier, car `cspell.json` le référence comme dictionnaire
utilisateur. Un fichier vide représente correctement l'absence actuelle de termes personnels et
évite de rendre la configuration cspell invalide.

Remplacer les noms d'organisation, de produits, de workspaces et les chemins métier détectés dans
`apple-notes`, `scripts/jdl` et `skill-manager` par des exemples neutres. Retirer les métadonnées
`author` qui nomment l'ancienne organisation, y compris dans les modèles produits par
`skill-manager`. Les exemples doivent continuer à démontrer exactement la même règle après
généricisation ; un exemple devenu inutile est supprimé plutôt que renommé artificiellement.

L'historique Git n'est pas réécrit. L'audit de #64 n'a trouvé aucun secret et la suppression dans
l'état courant répond au risque identifié : l'exposition de contexte opérationnel obsolète.

## Reconstruction de `skill-manager`

La version de `~/Code/brain` sert de référence sémantique, pas de source à copier. Le skill des
dotfiles conserve :

- `.agents/skills/` comme source canonique ;
- les adaptateurs `.claude/skills` et `.cursor/skills` existants ;
- les opérations `create`, `doctor`, `fix`, `cross-check` et `sync-index` ;
- `.agents/skills/README.md` comme index déterministe ;
- les catégories locales utilisées par cet index.

Les apports génériques à reprendre sont :

- la liste complète des champs de frontmatter agentskills.io, dont `allowed-tools`, avec les
  contraintes de type et de taille ;
- la distinction entre exigences du standard et conventions locales du dépôt ;
- la matrice des emplacements lus par les agents, adaptée au sens des symlinks de ce dépôt ;
- l'interdiction du shell à paramètres positionnels dans le corps d'un `SKILL.md`, où certains
  clients peuvent substituer `$0`, `$1` ou `$@` avant exécution ;
- la validation officielle par `skills-ref validate` lorsqu'elle est disponible, complétée par les
  contrôles locaux du doctor ;
- le principe qu'un routeur ne reçoit une règle que si une mesure démontre une mauvaise activation.

Les cinq procédures et `conventions.md` doivent être cohérentes entre elles. En particulier,
`doctor` doit accepter tous les champs autorisés, contrôler `Constraints` plutôt que `Rules`, et ne
pas présenter l'absence de routeur comme une faute. `create` ne doit pas produire de répertoire
absent de la structure canonique. `fix` doit accepter une évolution fonctionnelle explicitement
demandée, même si le skill courant passe déjà son doctor ; un finding reste obligatoire pour une
correction automatique non sollicitée.

Ne pas reprendre de Brain :

- `.claude/skills/` comme source canonique ;
- le routeur manuel propre à son coffre ;
- la suppression de `sync-index` et du README ;
- ses noms de notes, produits, packages ou équipes ;
- ses affirmations de validation non alignées avec son doctor actuel.

La reconstruction ne crée pas de nouvelle dépendance obligatoire. L'absence de `skills-ref` doit
être signalée comme une validation standard indisponible, puis les contrôles locaux continuent ;
elle ne doit ni installer un paquet implicitement ni permettre d'annoncer que le standard a été
validé.

## Découpage des commits

Deux commits indépendants sont prévus :

1. assainissement de #64 : suppressions AWS, Cursor et hooks, mise à jour Fish/Makefile/ADR,
   dictionnaire vidé et exemples hors `skill-manager` généricisés ;
2. reconstruction de `skill-manager`, généricisation de ses exemples et resynchronisation de son
   index.

Ce découpage permet de révoquer la reconstruction du skill sans réintroduire les configurations
obsolètes. Aucun nettoyage adjacent sans rapport ne rejoint ces commits.

## Vérification

La vérification doit couvrir les types de fichiers réellement modifiés :

- rechercher les noms, chemins, aliases et références supprimés dans l'arbre suivi ;
- vérifier que `gp` se développe en `git push`, qu'AWS reste dans le `Makefile`, que la cible Cursor
  ne déploie plus l'IDE et que `.cursor/` reste présent ;
- exécuter `bash -n` sur les scripts Bash conservés qui sont touchés ;
- faire analyser les fichiers Fish modifiés par `fish --no-execute` ;
- exécuter `make -n cursor` après inspection de ses dépendances, sans installer de logiciel ;
- valider chaque fichier Markdown et JSON modifié avec les barrières existantes du dépôt ;
- exécuter le doctor de `skill-manager`, puis `sync-index`, et vérifier que l'index ne dérive plus ;
- exécuter `skills-ref validate` si le binaire existe et nommer explicitement son absence sinon ;
- vérifier `git diff --check` et l'état final du worktree.

Les suppressions n'exigent pas de test d'installation réel : les recettes d'installation peuvent
muter Homebrew et `/Applications`, donc seules leur inspection et leur simulation `make -n` sont
autorisées dans ce chantier. Les résultats finaux nomment macOS comme environnement exercé ; Linux
n'est revendiqué que pour les validations purement syntaxiques effectivement exécutées.

## Clôture des issues

Après implémentation vérifiée et intégrée, commenter #54 avec la référence à #64 puis fermer #54.
Fermer #64 avec le résumé des deux commits et les validations exécutées. Ne fermer aucune issue sur
la seule base du design ou d'une branche non intégrée.
