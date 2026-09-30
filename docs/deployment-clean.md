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

| Surface                                                                                          | Action                                                                                                                               |
| ------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------ |
| Liens de configuration, instructions, règles, exécutables possédés, `.psqlrc`, wrapper Scrapling | Retirer uniquement le lien vers la source attendue ; conserver une destination divergente.                                           |
| Skills user Claude, Codex et Cursor                                                              | Lire les installations du manifeste Arnes, puis retirer les liens correspondants. Les skills tiers restent intacts.                  |
| Instructions Codex assemblées et définition d’agent copiée                                       | Supprimer le fichier régulier nommé, même édité localement.                                                                          |
| Deux thèmes Catppuccin de Bat et quatre dictionnaires Hunspell déployés                          | Supprimer seulement les fichiers réguliers nommés.                                                                                   |
| Hooks Arnes et remem                                                                             | Retirer les commandes et arguments identifiés, en conservant les groupes et handlers voisins.                                        |
| MCP user déclaré dans le manifeste                                                               | Retirer uniquement l’inscription dont la commande et les arguments correspondent ; retirer aussi son namespace de permissions Codex. |
| Include Git déployé                                                                              | Retirer seulement la valeur `~/.config/git/config.delta` avec le parser et l’écriture natifs de Git.                                 |

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

## Frontière encore bloquée : service remem

`remem-worker-install.ts` écrit `Library/LaunchAgents/dev.remem.worker.plist` et charge
`dev.remem.worker` dans le domaine GUI de l’utilisateur. Changer `$HOME` ne crée pas un domaine
launchd isolé. Le choix d’arrêter ou de conserver le service actif reste à préciser.

Tant que cette frontière n’est pas résolue, la présence du plist provoque un échec explicite avant
toute mutation ; le service, son plist et sa configuration restent conservés. Le CLI n’annonce
donc pas un nettoyage complet du profil minimal sur un poste où le worker est déployé.
La décision #152 est close ; son implémentation exhaustive n’est pas déclarée livrée.

## Preuves et limites

Les suites `clean-deployment.test.ts`, `clean-deployment-config.test.ts` et
`clean-deployment-decoding.test.ts` exercent le vrai CLI
dans des homes temporaires : absence, rejeu, destinations étrangères, ancêtres symlinkés,
arguments invalides, données conservées, configurations invalides, précision numérique,
octets UTF-8 invalides, normalisation des chemins et erreurs natives Git.

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
et `deployment-agent-handoff.test.ts` invoquent `make clean` dans un home temporaire : le lien vers
le binaire attendu disparaît, les fichiers, répertoires et liens étrangers sont conservés.

Ces observations portables ne remplacent pas la reconstruction complète du profil minimal sur
un runner macOS dédié, les diagnostics Arnes des seules ressources qu’ils couvrent ou la preuve
du service actif. Aucun nettoyage du home personnel ni installation globale n’est exécuté par ces
tests. Les limites des preuves Ubuntu et macOS sont indiquées dans la livraison et la CI.
