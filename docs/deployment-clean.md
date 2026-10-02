# Nettoyage explicite des déploiements

La décision [#152](https://github.com/SebastienElet/dotfiles/issues/152) porte sur les artefacts
des profils minimal et optionnel. Le nettoyage et l’installation restent deux opérations séparées.
`make clean` délègue à Moon ; les suppressions globales des données et caches Neovim ont été retirées.

## Inspection et application

Le CLI inspecte par défaut. Utiliser les chemins absolus du checkout et du home visé :

```sh
bun --config=/dev/null --no-env-file tooling/clean-deployment.ts /chemin/dotfiles /chemin/home
```

L’application demande `--apply`. La target publique `repository:clean` fournit ce drapeau
explicitement et vise `$HOME` :

```sh
moon run repository:clean
```

Cette commande ne relance pas l’installation. Après un nettoyage réussi, le profil minimal
est réinstallé séparément avec `moon run repository:install`. Le profil optionnel conserve son
entrée transitoire `make optional`. La reconstruction complète du poste est limitée à macOS.

## Périmètre possédé

| Surface                                                                                          | Action                                                                                                                                                               |
| ------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Liens de configuration, instructions, règles, exécutables possédés, `.psqlrc`, wrapper Scrapling | Retirer uniquement le lien vers la source attendue ; conserver une destination divergente.                                                                           |
| Skills user Claude, Codex et Cursor                                                              | Lire les installations du manifeste Arnes et reconnaître les anciens liens `remem-memory` Claude/Codex après retrait du manifeste. Les skills tiers restent intacts. |
| Instructions Codex assemblées et définition d’agent copiée                                       | Supprimer le fichier régulier nommé, même édité localement.                                                                                                          |
| Deux thèmes Catppuccin de Bat et quatre dictionnaires Hunspell déployés                          | Supprimer seulement les fichiers réguliers nommés.                                                                                                                   |
| Hooks Arnes et remem                                                                             | Retirer les commandes et arguments identifiés, en conservant les groupes et handlers voisins.                                                                        |
| MCP user actuel ou ancien déploiement remem                                                      | Retirer uniquement l’inscription dont la commande et les arguments correspondent ; retirer aussi son namespace de permissions Codex.                                 |
| Include Git déployé                                                                              | Retirer seulement la valeur `~/.config/git/config.delta` avec le parser et l’écriture natifs de Git.                                                                 |

Les sources versionnées, paquets et outils tiers, plugins Fisher/TPM, caches runtime, données
d’applications, bases remem, historiques et identifiants sont conservés. Les politiques
`autoMemoryEnabled` et `features.memories` ne sont pas restaurées : aucun état antérieur n’est connu.
Les répertoires parents restent en place. Le nettoyage n’efface jamais récursivement un répertoire
d’application et ne suit jamais la cible du lien supprimé.

Les JSON partagés sont validés avant réécriture et remplacés par renommage local ; leurs valeurs
étrangères restent présentes. Les lexèmes numériques sont conservés par le reviver et
`JSON.rawJSON` natifs, vérifiés avec Bun 1.4.0 ; aucune conversion numérique n’est réécrite dans
le document. Des octets UTF-8 invalides provoquent un refus avant toute mutation.
Les blocs TOML possédés sont retirés uniquement si le parser natif confirme exactement
le résultat attendu, sans changement des autres valeurs. Une représentation non prise en charge
reste intacte et provoque un échec explicite. Les configurations partagées symlinkées sont refusées.

Un artefact absent est un succès, y compris lors d’un second passage. Une erreur d’inspection,
de validation ou de mutation produit un statut non nul. Les mutations déjà réalisées ne sont pas
annulées globalement ; les erreurs d’application sont signalées. Un processus concurrent qui
modifie les chemins pendant le nettoyage n’est pas couvert par une garantie d’atomicité globale.

## Retrait du worker remem

Le [retrait autorisé le 2026-10-01](remem.md) supprime remem des profils Codex et Claude.
Le nettoyage reconnaît ses anciennes inscriptions MCP, ses hooks et ses liens même après leur
retrait du manifeste. Il conserve les bases, clés, logs et exécutables tiers, ainsi que
`agent-handoff` et la mémoire historique de Cursor.

Si le plist ou le lien de configuration remem possédé existe, le CLI lit le plist avec
`plutil -convert json` et inspecte le domaine utilisateur par l’API publique
[`SMCopyAllJobDictionaries`](<https://developer.apple.com/documentation/servicemanagement/smcopyalljobdictionaries(_:)>).
Il exige le plist canonique et les `Label`, `ProgramArguments` et éventuel `Program` attendus du
service chargé. Un plist étranger, une identité étrangère, une snapshot indisponible, incomplète,
ambiguë ou de forme inconnue provoque un refus avant toute mutation.

Cette API est dépréciée ; sa disponibilité et les dictionnaires observés ne constituent pas un
contrat pour les futures versions de macOS. L’inspecteur JXA s’exécute dans un processus natif
jetable : aucune gestion autonome des allocations CF de longue durée n’est revendiquée.
La sortie non contractuelle de `launchctl print` n’est pas parsée.

Après validation des configurations partagées et de tous les chemins, le CLI revalide l’identité,
exécute `launchctl bootout --wait` sur le seul service reconnu du domaine GUI de l’UID courant,
puis exige son absence dans une nouvelle snapshot avant de supprimer le plist et la configuration.
Le home concerné doit appartenir à cet UID ; un appel sous `sudo` visant le home d’un autre utilisateur est refusé avant les mutations.
Chaque commande native dispose d’un timeout de 15 secondes ; un échec ou timeout conserve les
artefacts restants et produit un statut non nul. Aucun code d’erreur launchctl n’est assimilé à
une absence. L’inspection sans `--apply` n’arrête aucun service.

Un home temporaire ne crée pas de domaine launchd isolé. Le test natif utilise un label unique,
un exécutable inoffensif de fixture et une vérification préalable de son absence ; il ne cible
jamais le worker personnel. Les commandes de profil n’installent plus de worker remem.

## Preuves et limites

Les suites `clean-deployment.test.ts`, `clean-deployment-config.test.ts` et
`clean-deployment-decoding.test.ts` et `clean-deployment-remem.test.ts` exercent le vrai CLI
dans des homes temporaires : absence, rejeu, destinations étrangères, ancêtres symlinkés,
arguments invalides, données conservées, configurations invalides, précision numérique,
octets UTF-8 invalides, normalisation des chemins, erreurs natives Git et cycle de retrait du
worker avec des providers de fixture. `clean-deployment-worker-inspect.test.ts` exerce la sélection
et les refus du véritable script JXA avec des snapshots substituées.

Sur macOS, `tooling:worker-native-test` exerce le module de retrait avec une spécification immuable
de fixture : vrai LaunchAgent en cours, signal de fin alors que la configuration existe encore,
PID disparu, plist supprimé, snapshot absente et rejeu. Cette preuve est exécutée dans le job
macOS de la CI de déploiement existante ; Ubuntu conserve les tests portables sans simuler une
preuve du service macOS.

`clean-deployment-moon.test.ts` exerce la target publique de nettoyage et la réinstallation de
déploiements Moon sélectionnés du profil minimal, sans leurs dépendances globales. Une preuve
distincte couvre les liens Cursor, PostgreSQL et Scrapling du profil optionnel ; elle n’installe
pas leurs applications ou conteneurs tiers. Les fichiers Fisher déjà présents dans les sources
restent conservés après suppression du lien Fish. Le snapshot des artefacts exclut les caches
runtime, dont `~/.bun/install/cache` ; les fichiers de ce cache sont comparés avant et après le
nettoyage en octets exacts, avec une sentinelle étrangère. Pour ces fixtures, le cache actif du
transpiler Bun est isolé dans un dossier frère du home via
[`BUN_RUNTIME_TRANSPILER_CACHE_PATH`](https://bun.sh/docs/runtime/environment-variables#runtime-transpiler-caching) :
ses écritures `.pile` ne sont pas attribuées au nettoyage. Les assertions de conservation du cache
étranger restent inchangées. Les tests `deployment-agent-memory.test.ts`
et `deployment-agent-handoff.test.ts` invoquent `repository:clean` dans un home temporaire : le lien vers
le binaire attendu disparaît, les fichiers, répertoires et liens étrangers sont conservés.

Ces observations portables ne remplacent pas la reconstruction complète du profil minimal sur
un runner macOS dédié, les diagnostics Arnes des seules ressources qu’ils couvrent ou une preuve
de compatibilité avec toutes les futures versions du service natif. Aucun nettoyage du home personnel ni installation globale n’est exécuté par ces
tests. Les limites des preuves Ubuntu et macOS sont indiquées dans la livraison et la CI.

Le job existant « Minimal macOS profile » exécute `tooling:smoke-minimal` : installation complète,
rejeu silencieux, vraie target `repository:clean`, réinstallation complète et nouveau rejeu silencieux.
Les vérifications natives des paquets, exécutables et snapshots existants s’appliquent avant et
après reconstruction. Le smoke ne suppose plus qu’un profil sans remem crée `.claude.json`.
Cette installation globale reste réservée au runner CI jetable ; elle n’est pas exécutée localement
avec un home de fixture.
