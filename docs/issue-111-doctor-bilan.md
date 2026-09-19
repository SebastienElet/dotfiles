# #111 — Bilan de Doctor et travaux restants

D1 et D2 ont été approuvées le 19 septembre 2026. La
[PR #343](https://github.com/SebastienElet/dotfiles/pull/343) contient maintenant le
correctif D1 : Doctor ne lance plus le résolveur Codex. L'inventaire actif des plugins
est explicitement indisponible ; exposition locale, politique et skills système
restent diagnostiqués. D2 conserve les défauts MCP et limite l'équivalence aux
sélections explicites identiques.

La référence utilisateur reste [arnes-doctor.md](arnes-doctor.md). Les
[brouillons S1–S8](issue-111-sync-brouillons.md) concernent les synchronisations
futures, sans leur implémentation. #111 reste ouverte jusqu'à disposition de ses
critères restants et intégration du correctif ; aucune fusion n'est effectuée ici.

## Livraisons et autorités réutilisées

- [#123 / PR #288](https://github.com/SebastienElet/dotfiles/pull/288) livre les dix
  familles et l'agrégation ; ses tests sont réutilisés.
- [#126 / PR #338](https://github.com/SebastienElet/dotfiles/pull/338) livre la
  référence commune ; seules les descriptions du résolveur devenues fausses sont corrigées.
- [#124 / PR #339](https://github.com/SebastienElet/dotfiles/pull/339) atteste le
  graphe/recettes Moon, les fixtures 0/1/2 et les plateformes.
  Son [job Ubuntu 24.04](https://github.com/SebastienElet/dotfiles/actions/runs/35072676955/job/104717640460)
  a réellement exécuté cinq gates et 742 tests. Ce résultat reste historique.
- Les ADR acceptées [001](adr/001-makefile-installateur.md),
  [003](adr/003-deploiement-par-symlinks.md),
  [023](adr/023-ci-lint-et-installation.md),
  [038](adr/038-frontieres-home-harness-tooling.md) et
  [041](adr/041-frontiere-automatisation-typescript-rust.md) restent inchangées.
  D1 réduit l'observation Codex permise par #166/#168 ; elle ne transfère aucune
  propriété de plugins à Arnes. L'[ADR-043](adr/043-telemetrie-arnes-minimale-et-bornee.md)
  continue de distinguer configuration et résultat de session.

## Rattachements et couverture par famille

Le relevé REST du 16 septembre a vérifié les 15 enfants fermés de #111 :
#109, #110, #112–#115, #117–#124 et #126. Aucun rattachement ne manquait.
Les clôtures manuelles de #119/#120 ont été distinguées des PR automatiquement liées.
Le [bilan initial](https://github.com/SebastienElet/dotfiles/blob/bc210067e6bccccee3efc9c4fd92039d2e801842/docs/issue-111-doctor-bilan.md)
conserve la correspondance complète famille/livraison/représentation.

La matrice actuelle des représentations se trouve dans la référence Doctor :
les trois agents n'ont pas les mêmes capacités. La présence de fixtures Claude,
Cursor et Codex ne prouve pas le chargement de leur configuration en session.

## Bilan des neuf critères

| Critère                                       | État et preuve                                                                                                                                                                                                            |
| --------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1. Sous-issues Doctor rattachées              | Vérifiées : les 15 enfants et leurs livraisons existent.                                                                                                                                                                  |
| 2. Sélecteurs, filtres et états isolés        | Suites de ressources réutilisées, plus les compléments macOS ci-dessous. Unsupported et sélection vide ne valent pas conformité.                                                                                          |
| 3. Agrégat ordonné et diagnostics individuels | Tests agrégés conservés ; D2 qualifie l'équivalence à agent/scope explicites identiques. Les défauts MCP ne changent pas.                                                                                                 |
| 4. Human/JSON et sorties                      | Contrat partagé conservé : 0 healthy/unsupported/vide, 1 drift, 2 error ; erreurs d'arguments/sortie possibles hors rapport JSON.                                                                                         |
| 5. Lecture seule et non-exécution             | Résolveur Codex supprimé ; régression observée rouge puis verte sur direct/agrégat, human/JSON et configurations absente/vide/activée. Le témoin externe complète les snapshots, sans leur donner une portée universelle. |
| 6. Installation et référence commune          | Preuves Moon de #339 réutilisées ; binaire/manifeste restent livrés par les mêmes recettes. Référence #126 alignée sur la réduction d'inventaire.                                                                         |
| 7. Contrôles Rust                             | Cinq gates Moon réussies sur le correctif local macOS : fmt, Clippy, check, test, doc ; 713 tests réussis, 0 échec, 0 ignoré. La CI du nouveau head est à vérifier séparément.                                            |
| 8. Plateformes/représentations explicites     | Correctif exécuté localement sous macOS 26.6.2 arm64, Cargo 1.98.0, réseau interdit. La preuve Ubuntu de #339 concerne l'ancien code ; aucune session d'agent réelle n'est revendiquée.                                   |
| 9. Synchronisations ultérieures               | Huit brouillons distincts ; hooks renvoie à setup existant, manifest à Moon. Publication et rattachements nécessitent validation. Pas de sync dans ce correctif.                                                          |

## Correctif D1 et preuve ciblée

Le chemin [Codex](../tooling/arnes/src/skills/external/codex.rs) ne lit plus que
la configuration locale pour les plugins. Les modules de subprocessus, de lecture
des réponses et de sélection d'artefacts ont été retirés.

Le test [doctor_read_only](../tooling/arnes/tests/doctor_read_only.rs) installe un
faux Codex qui écrirait un témoin hors de HOME/projet et créerait puis supprimerait
un fichier dans HOME. Avant correction, il échoue sur ce témoin malgré des snapshots
identiques. Après correction, les 12 appels passent : direct et agrégé, human et JSON,
configuration absente, vide et avec plugin activé. Aucun vrai agent n'est lancé.

Les [tests de plugins](../tooling/arnes/tests/external_codex_plugins.rs) vérifient
les états enabled/disabled/absent, la politique, l'ordre des deux formats et l'absence
d'inférence depuis un cache solitaire. Avant correction, le cas enabled échouait
car il annonçait une disponibilité ; il conserve désormais exposition configurée
et activation inconnue. Un plugin activé hors politique reste drift.

Les dix tests propres aux réglages des skills système restent présents dans
[external_codex_skill_config](../tooling/arnes/tests/external_codex_skill_config.rs).
Les tests de timeout, sorties du résolveur et artefacts sélectionnés ont été retirés
avec ce mécanisme. La baisse du nombre total de tests inclut aussi les répétitions
du support de fixture compilées dans les suites supprimées ; elle ne représente
pas une baisse des checks de configuration locale.

| Garantie antérieure                                                     | Disposition D1                                                          |
| ----------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| Sélection active autoritative, artefact/version/chemin/skills du plugin | Indisponibles explicitement ; aucune sélection depuis le seul cache.    |
| Timeout, taille/schéma des réponses, cohérence config/résolveur         | Sans objet : aucun résolveur ni réponse externe n'est exécuté/consommé. |
| Exposition configurée et politique                                      | Conservées ; disponibilité non déduite d'enabled.                       |
| Erreurs de configuration, redaction et frontières des skills système    | Suites existantes conservées.                                           |
| Ordre human/JSON et lecture seule                                       | Adaptés à la configuration locale ; témoin externe plus snapshots.      |

Validation locale du correctif : Moon 2.5.4, sous
`sandbox-exec -p '(version 1)(allow default)(deny network*)'`,
`moon exec --upstream none --no-actions --force arnes:fmt arnes:clippy arnes:check arnes:test arnes:doc`.
Les cinq cibles ont été exécutées, avec 713 tests réussis. L'interdiction réseau
appartient à l'environnement de preuve ; la correction supprime le mécanisme
d'exécution externe dans le code possédé, sans ajouter de sandbox à Arnes.

## Compléments exécutés, sans refaire les suites

Au 16 septembre, sur `272e8a6`, six appels CLI macOS en human/JSON avaient complété
les cas positifs instructions Claude project (`@AGENTS.md`) et MCP user Cursor/Codex.
Ils sortaient 0 avec un diagnostic healthy. Ces observations restent datées ;
elles ne sont pas une preuve Linux ni une session réelle d'agent.

### Contre-exemple de lecture seule

Le probe historique sur `272e8a6` lançait deux commandes Codex par appel Doctor,
avec sortie 0 et snapshots identiques malgré un témoin hors des arbres observés.
Ce défaut est désormais reproduit par la régression puis corrigé dans la PR.
Les snapshots seuls restent insuffisants pour exclure les effets temporaires.

### Contre-exemple d'équivalence sans scope MCP

Avec une seule déclaration Claude MCP project et sa configuration absente,
`doctor mcp --agent claude` rend 0/unsupported user ; l'agrégat rend 1/drift project.
Avec `--scope project`, les diagnostics MCP sont identiques. D2 conserve ce
comportement documenté ; elle ne prétend pas rendre les appels sans scope équivalents.

## Suite

La validation puis la publication des sous-issues restent distinctes de l'intégration
du correctif. Le contrat MCP des trackers doit reprendre D2. Une fois ces dispositions,
les preuves du head final et l'intégration établies, #111 peut être clôturée sans
attendre l'implémentation des synchronisations futures.

Auteur `/root`, auditeur `/root/audit_doctor_claims` ;
[registre complet des claims](issue-111-doctor-audit.md). Commentaires de code ajoutés : aucun.
