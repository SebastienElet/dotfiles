# Nettoyage des déploiements

## Usage

Inspecter puis retirer les artefacts possédés des profils minimal et optionnel,
selon l'[ADR-003](adr/003-deploiement-par-symlinks.md). La reconstruction complète
du poste est limitée à macOS. Préserver les fichiers personnels avant application.

## Procédure

Depuis le checkout canonique, inspecter avec les chemins absolus du dépôt et du HOME :

```sh
bun --config=/dev/null --no-env-file tooling/clean-deployment.ts /chemin/dotfiles /chemin/home
```

La commande inspecte par défaut. Examiner les lignes `would-remove` et `would-update`.
La target suivante applique les suppressions au HOME courant :

```sh
moon run repository:clean
```

Elle ne réinstalle rien. Relancer séparément `moon run repository:install`, puis
`moon run repository:optional` si nécessaire. Ne pas exécuter ces installations
globales depuis un worktree ou un HOME de fixture.

## Limites et vérification

Les liens ne sont retirés que s'ils ciblent la source attendue. En revanche, les
instructions Codex assemblées, la définition d'agent copiée, les thèmes Bat et les
dictionnaires Hunspell nommés sont supprimés même après modification locale.
Les inscriptions MCP, hooks et includes Git reconnus sont retirés en conservant
les voisins. Les sources, paquets tiers, données, historiques, secrets et caches
runtime restent conservés ; les répertoires parents ne sont pas effacés.

Le retrait historique remem reconnaît aussi ses anciens artefacts hors manifeste.
Il arrête seulement le worker dont l'identité native est reconnue avant de retirer
ses fichiers. Une identité inconnue ou une inspection impossible provoque un refus.
Les bases remem et les réglages de mémoire désactivée restent conservés.

Une absence est un succès rejouable. Une erreur d'inspection, de parsing ou de
mutation retourne un statut non nul ; les mutations précédentes ne sont pas
annulées globalement. Les configurations partagées symlinkées sont refusées.

`tooling:deployment-test` exerce les fixtures portables sur macOS et Ubuntu ;
`tooling:worker-native-test` exerce le service natif macOS. Le smoke minimal de CI
vérifie nettoyage et réinstallation sur un runner macOS dédié. Ces preuves ne
valident pas une suppression sur votre HOME personnel.
