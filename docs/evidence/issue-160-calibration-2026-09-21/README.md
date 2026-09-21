# Preuves de calibration #160

Quatre exécutions synthétiques, Codex CLI 0.154.0, `gpt-5.6-sol` demandé, effort `medium`,
macOS 27.0 arm64. [Analyse et limites](../../issue-160-calibration.md).

- `manifest.json` : révision des sources, skills, environnement, invocation, delta exact, ordre
  des conditions, empreintes des raws et transformations appliquées aux copies publiables.
- `fixtures.json` : prompts UTF-8 et contenu exact des deux fixtures de départ. Reconstituer les
  fichiers dans des dépôts neufs ; les sources du harnais proviennent du commit du manifeste.
- `input-hashes.json` : empreintes des 246 fichiers figés, y compris les snapshots initiaux du
  checkpoint, du protocole et de l'inventaire. Ces trois snapshots sont dans l'archive locale ;
  les documents courants ont ensuite reçu un lien vers les résultats.
- `grading-plan.json` et `scores-blinded.json` : grille préalable et scores enregistrés avant
  lecture de la correspondance des conditions ; aucune réécriture des scores après levée du masque.
- `summary.json` : jointure explicite avec les conditions, résultats, consommation, limites et
  relevés du quota partagé. Ce n'est pas un rapport au format Arnes.
- `run-01` à `run-04` : JSONL natifs avec chemins et identifiants normalisés, stderr, réponse
  finale, diff, durée/code processus ; P1 conserve aussi le test natif indépendant après exécution.
  `change.json` encode le diff exact dans une chaîne JSON afin de conserver ses espaces de
  contexte sans introduire de whitespace terminal dans les fichiers du dépôt.

La condition A garde le texte actuel ; B retire A039.a. Les autres contenus sont identiques.
La grille distingue succès de la tâche et facettes de restitution. Les fixtures README sont
synthétiques ; leurs contrats et leurs résultats ne décrivent aucun projet réel de l'utilisateur.

Les originaux exacts et tous les snapshots sont conservés dans le dossier local
`issue160-calibration-20260921` remis avec cette tâche. Le répertoire contient les observations
de préflight Node 24 et Node 26 ainsi que l'écart constaté sur le shell, sans authentification.
Les exports JSONL ne sont pas octet pour octet les originaux : `manifest.json` relie leurs hashes
et décrit les remplacements. Les chemins `/synthetic/…` des copies sont des identifiants,
pas des destinations à utiliser telles quelles.

Pour rejouer, conserver le même commit, les mêmes skills et prompts, reconstruire les projections
globales dans un home neuf et rétablir un environnement équivalent. La commande native et ses
options figurent dans le manifeste ; ne pas copier ses chemins symboliques sans adaptation.
La vérification du shell réel est requise : le runtime des commandes agent était Node 26.8.1,
alors que la préparation utilisait Node 24.21.0. Chaque nouvelle exécution reçoit un nouvel ID,
ses propres logs et ses limites ; elle ne remplace pas une observation conservée.

Ces fichiers ne forment ni un runner, ni un schéma de registre, ni une nouvelle gate. Les JSON
peuvent être lus par un parseur natif ; les preuves comportementales sont les tests et les sorties
enregistrées, pas la validité syntaxique des rapports.
