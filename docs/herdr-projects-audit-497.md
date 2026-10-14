# Herdr Projects : lanceur et skills

Audit du 14 octobre 2026 pour [#497](https://github.com/SebastienElet/dotfiles/issues/497),
depuis le worktree `hp/dotfiles/t-0001-497-lanceur-et-skills-avec-herdr-project`,
base `ad9fb8fe`. Résultat : conserver les 49 skills possédés, adapter trois frontières
avec Projects et conserver le lanceur au clic tant que son abandon n'est pas décidé.
Aucune mesure de fréquence d'usage ou de gain de temps n'a été réalisée.

## Sources et portée

- ADR acceptées [025](adr/025-agents-md-source-unique.md),
  [028](adr/028-skills-ssot.md), [029](adr/029-pas-de-skills-tierces.md),
  [038](adr/038-frontieres-home-harness-tooling.md) et
  [040](adr/040-skills-user-dans-harness.md) : instructions canoniques,
  frontières et collections possédées. Aucune ne prescrit de conserver ce lanceur.
- Sources actuelles : [lanceur](../tooling/herdr-issue-worktree/README.md),
  son manifest, `dispatch.ts`, `dispatch-lock.ts`, `sessions.ts`, `prompts.ts`,
  `register-work.ts`, son compagnon Rust et leurs tests ;
  [skill associé](../harness/skills/herdr-issue-worktree/SKILL.md).
  Les observations natives historiques documentées concernent Herdr 0.9.3,
  pas une exécution du parcours sous 0.9.4 durant cet audit.
- Herdr Projects installé : `0.2.34+4e4548c.1790629723`, checkout du plugin
  `4e4548c3c43888e6be1c96906215998f07dc63f0`, lu sans modification.
  Sources officielles de cette version :
  [opérations](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/docs/operations.md),
  [création et reprise](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/threads.rs),
  [transport natif](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/herdr.rs),
  [livraison du brief](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/brief.rs),
  [consignes du coordinateur](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/skill/COORDINATOR.md)
  et [du thread](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/skill/THREAD.md).
  La documentation décrit un mécanisme ; elle ne prouve pas son exécution ici.
- Les 45 `harness/skills/*/SKILL.md` et les quatre `.agents/skills/*/SKILL.md`
  ont été lus. Leurs README sont des index dérivés, pas des copies maintenues.
  `home/.arnes.yaml` déclare les installations user ;
  `tooling/install-agent-skills.ts` déploie des liens feuille vers ces sources.
  `.claude/skills`, `.cursor/skills` et `.codex/skills` sont des adaptateurs projet.
  Les skills système, `herdr`, `autoproject` et ceux d'autres plugins sont exclus.

## Comparaison des parcours

Ce tableau compare les mécanismes lus dans les sources, sauf preuve explicitement nommée.
« Conservé » signifie que la capacité reste déclarée dans son parcours ; cela ne
certifie pas une équivalence de protection entre les deux plugins.

| Capacité ou protection      | Lanceur au clic                                                                                                                   | Herdr Projects                                                                                                                       | Conséquence                                                                                                           |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------- |
| Entrée tracker              | Ctrl-clic GitHub/GitLab/Linear ; titre authentifié par l'agent de préparation                                                     | Tâche et titre fournis au thread ; pas de `link_handlers` dans le manifest installé                                                  | Le raccourci tracker reste propre au lanceur                                                                          |
| Agent                       | Choix explicite Claude/Codex                                                                                                      | Profil autorisé, plusieurs types d'agents, profils distants                                                                          | Projects remplace le choix et lancement pour ses tâches                                                               |
| Isolation                   | Worktree lié, identité Git et cwd vérifiés ; checkout principal intact                                                            | Worktree par défaut avec repo, branche `hp/<project>/<thread>-…` ; modes tab et checkout explicites aussi disponibles                | Conserver le mode worktree pour implémenter ce dépôt ; ne pas confondre tous les modes avec l'isolation               |
| Branche et reprise d'une PR | Workflow du dépôt, `linear-start`/`linear-workflow` pour Linear/Bitbucket                                                         | Branche de thread générée ; `--base` explicite ou base découverte                                                                    | Les politiques tracker et la reprise d'une branche de PR ne sont pas remplacées par la création d'un thread           |
| Reprise                     | Bindings durables par issue, repo et rôle ; contrôles d'occupant et de session ; agent actif conservé, continuation idle manuelle | `thread restart` réutilise le pane ou rouvre le worktree enregistré ; refuse notamment un thread actif ou résolu ; adoption possible | Projects gère ses threads, pas les bindings du lanceur                                                                |
| Absence de doublon          | Réservation exclusive par session/issue ; collisions et incertitude bloquent une création                                         | Allocation d'un ID par nouveau thread ; consigne de transférer un suivi au thread existant                                           | Ni registre commun ni déduplication par identité tracker entre parcours ; ne pas lancer la même affectation deux fois |
| Focus et occupants          | `--no-focus`, snapshots et refus des occupants incompatibles                                                                      | Création/ouverture worktree avec `--no-focus` dans `herdr.rs` ; commandes de navigation distinctes                                   | Intention de préserver le focus dans les deux créations ; aucune nouvelle preuve native comparative                   |
| Contexte                    | URL et contexte complets dans JSON privé ; référence courte encodée dans le prompt initial                                        | Brief Markdown avec objectif, instructions, mémoire, tâche et chemins ; ligne courte de lecture au démarrage                         | Projects remplace le briefing de ses threads, pas la validation des exigences                                         |
| Échec ou timeout            | Ressources et réservations incertaines conservées ; pas de relance aveugle                                                        | Échec enregistré ; restart selon état ; livraison observe écran/readiness et compte les essais avant émission                        | Politiques de reprise distinctes, pas équivalence de livraison garantie                                               |
| Trust et autorisations      | Aucun dialogue trust répondu ; le provisioning n'autorise pas commit/push/merge                                                   | Paramètres safety et profils ; trust peut être confié au coordinateur ; la documentation qualifie ces protections de soft            | Aucun changement de trust, permissions ou safety dans cette PR                                                        |
| Suivi et livraison          | État de préparation et handles ; développement assuré par les workflows composés                                                  | Report, Next, Remember, inbox, progression, suivi GitHub des PR et nettoyage après résolution                                        | Projects remplace la coordination et le suivi de ses tâches ; les skills de développement/revue restent nécessaires   |
| Nettoyage                   | Plugin indépendant `dotfiles.worktree-cleanup`, refus Git sans force                                                              | Copie du report/library puis résolution et retrait du worktree, branche si PR mergée                                                 | Conserver les deux mécanismes dans leur portée ; aucun nouveau retrait exécuté ici                                    |

Le worktree et le brief fournis à cet audit sont observables. Ils ne prouvent
ni la livraison initiale du prompt par le ticker ni la reprise native d'un agent.
Le signalement de progression a répondu `not inside a Herdr pane; nothing reported`.

## Bilan des skills possédés

Les consommateurs sont les agents déclarés dans `home/.arnes.yaml`, leurs demandes
explicites et les workflows qui composent ces skills. L'absence d'une référence
textuelle n'est pas une preuve d'absence d'usage. Les décisions ci-dessous évaluent
la fonction encore nécessaire, pas sa fréquence. Tous les skills restent présents.

| Skill user                      | Décision  | Fonction conservée ou adaptation                                                        |
| ------------------------------- | --------- | --------------------------------------------------------------------------------------- |
| `agent-instructions`            | Conserver | Sources, discovery et projections des instructions                                      |
| `assertledger`                  | Conserver | Qualification et rejeu d'une régression demandée                                        |
| `claude-developer`              | Conserver | Handoff manuel vers Claude ; compose `development`                                      |
| `code-enforcement`              | Conserver | Conception des refus et des chemins de bypass                                           |
| `code-review`                   | Conserver | Revue du diff contre standards et exigences                                             |
| `code-simplify`                 | Conserver | Simplification du code avec contrat observable                                          |
| `codebase-design`               | Conserver | Cohésion, interfaces et frontières des invariants                                       |
| `design-claim-audit`            | Conserver | Audit indépendant des garanties documentées                                             |
| `development`                   | Conserver | Implémentation Claude, checks et revue Codex via `review-fix`                           |
| `diagnosing-bugs`               | Conserver | Reproduction et diagnostic des échecs                                                   |
| `disaster-recovery-plan`        | Conserver | PRA DOCX et collecte des preuves                                                        |
| `grill-me`                      | Conserver | Entrée manuelle d'interview ; compose `grilling`                                        |
| `grilling`                      | Conserver | Résolution des décisions matérielles par dépendances                                    |
| `handoff`                       | Adapter   | Écrire le report Projects requis avant le prompt de reprise et l'arrêt                  |
| `harness-reflection`            | Conserver | Échecs répétés et amélioration du harness fondée sur essais                             |
| `harness-simplify`              | Conserver | Diagnostic, conséquences des suppressions et consommateurs                              |
| `herdr-issue-worktree`          | Adapter   | Garder le lancement standalone ; exclure une affectation Projects déjà provisionnée     |
| `improve-codebase-architecture` | Conserver | Exploration manuelle des frictions architecturales                                      |
| `issue-creation`                | Conserver | Cohérence, doublons, autorité et relecture des issues publiées                          |
| `issue-simplify`                | Conserver | Simplification ciblée d'issues                                                          |
| `linear-issue-spec`             | Conserver | Parcours produit, états visibles et critères fonctionnels                               |
| `linear-start`                  | Conserver | Éligibilité, reprise de PR et lifecycle Linear/Bitbucket                                |
| `linear-sync`                   | Conserver | Réconciliation tracker/forge avec réalité des PR                                        |
| `linear-workflow`               | Conserver | Invariants partagés et preuve de completion                                             |
| `memory-governance`             | Adapter   | Séparer admission personnelle et candidats `Remember` transmis au coordinateur          |
| `obsidian-retrieval`            | Conserver | Retrieval local sourcé en lecture seule                                                 |
| `output-discipline`             | Conserver | Présentation explicitement activée ; format report imposé par le brief reste applicable |
| `para-organizer`                | Conserver | Organisation PARA des fichiers personnels                                               |
| `pr-feedback`                   | Conserver | Collecte factuelle des retours de reviewers                                             |
| `pr-fix`                        | Conserver | Réparation bornée de PR avec revue indépendante                                         |
| `pr-verdict`                    | Conserver | Verdict demandé sur SHA et état réel de la forge                                        |
| `prepare-next`                  | Conserver | Sélection du ticket éligible et préparation des décisions                               |
| `proof-integrity-review`        | Conserver | Audit de la pertinence des mécanismes de vérification                                   |
| `prose-edit`                    | Conserver | Révision préservant sens et voix                                                        |
| `requirements-clarification`    | Conserver | Résolution des politiques matérielles manquantes                                        |
| `review-fix`                    | Conserver | Corrections locales et revue finale indépendante                                        |
| `security-assurance-plan`       | Conserver | PAS DOCX avec faits et preuves traçables                                                |
| `skill-manager`                 | Conserver | Maintenance des skills, conventions et index dérivés                                    |
| `skill-simplify`                | Conserver | Simplification ciblée d'un skill avec comportements préservés                           |
| `tdd`                           | Conserver | RED/GREEN pour comportement exécutable possédé                                          |
| `things-tasks`                  | Conserver | CLI Things 3, indépendant des projets Herdr                                             |
| `to-spec`                       | Conserver | Synthèse manuelle en spécification fonctionnelle                                        |
| `visual-explanation`            | Conserver | Visualisation utile avec inspection du rendu                                            |
| `wayfinder`                     | Conserver | Carte canonique des décisions ; utilise déjà le format projet existant                  |
| `workflow-automation`           | Conserver | Automatisation d'un workflow répété et démontré                                         |

| Skill projet | Décision  | Fonction conservée                                        |
| ------------ | --------- | --------------------------------------------------------- |
| `adr-record` | Conserver | Enregistrement explicite d'une décision actée             |
| `dotfiles`   | Conserver | Moon, symlinks, profils et frontières d'installation      |
| `neovim`     | Conserver | Configuration LazyVim et validation native                |
| `scripts`    | Conserver | Frontières shell/TypeScript/Rust et vrais points d'entrée |

`wayfinder/references/map.md` respecte déjà un format projet existant : créer une
nouvelle carte concurrente n'est pas demandé. `development` possède la fin du
développement et sa revue, que le suivi d'une PR par Projects ne remplace pas.
Le Stop hook de `agent-handoff` reste indépendant de Projects.

## Changements et décision restante

- Trois adaptations de consignes dans les sources user, avec scénarios existants
  complétés pour `herdr-issue-worktree` et `memory-governance`. Aucune nouvelle gate.
- README racine et du lanceur : Projects pour les tâches coordonnées, lanceur pour
  l'entrée standalone. Les consignes n'empêchent pas techniquement un utilisateur
  de Ctrl-cliquer la même issue : il n'y a pas de registre partagé ajouté.
- Manifest Arnes, tâches Moon, adaptateurs et nettoyage conservés : aucun retrait
  de capacité ou de déploiement démontré. L'index user est régénéré depuis le
  frontmatter ; les premières phrases et les membres restent identiques.

**Décision de Sébastien avant un éventuel retrait :** accepter de perdre l'entrée
Ctrl-clic GitHub/GitLab/Linear, la lecture du titre authentifié au lancement et la
reprise par bindings d'issue hors Projects, et passer ces demandes au coordinateur
ou aux commandes Projects. L'issue n'autorise pas implicitement cette perte.
Recommandation actuelle : conserver le lanceur comme parcours standalone.
Ne pas confondre cette décision avec un retrait de règles de développement.

## Vérification et limites

Environnement local : macOS ARM64, Herdr CLI 0.9.4, Projects 0.2.34,
Bun 1.4.3 ; Moon disponible 2.6.0, dépôt déclarant 2.5.3.
Les nouveaux fichiers sont indexés avant les contrôles. `.semctx/` est absent
du worktree ; aucun fichier sémantique ni checkout principal initialisé.

Contrôles locaux exécutés :

| Contrôle                                                                             | Résultat et portée                                                                                                                                          |
| ------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `tooling/check-prettier.ts --check '**/*.{yml,yaml,md,json}'`                        | PASS, fichiers indexés et Markdown des skills                                                                                                               |
| `moon run repository:cspell-check --upstream none --no-actions`                      | PASS, inputs de la tâche et 109 Markdown de skills ; dépendances d'installation exclues                                                                     |
| `cargo run --locked --manifest-path tooling/arnes/Cargo.toml -- eval validate-evals` | PASS, 2 cas comportementaux et 31 contrats d'activation valides ; vérification de format, pas exécution des agents                                          |
| `cargo build --release --locked` puis `cargo test --locked`, manifest du lanceur     | PASS, compagnon construit et 6 tests Rust                                                                                                                   |
| `bun test tooling/herdr-issue-worktree tooling/install-agent-skills.test.ts`         | PASS, 145 tests, 467 assertions ; mécanismes du lanceur conservé et déploiement en fixtures                                                                 |
| Installateur existant avec trois destinations temporaires                            | PASS, six liens déclarés des trois skills adaptés lisibles et résolus vers ce worktree ; fixtures supprimées                                                |
| `skill-manager doctor` appliqué manuellement aux trois skills                        | Frontmatter, corps, ressources locales, placeholders, evals présents et index conformes avant/après ; `skills-ref` absent, validation standard indisponible |
| `sync-index user`, deux générations                                                  | Identiques entre elles et à l'index existant, SHA-256 `4d7b7e1a2a61c7390710411d11e4a0504082d8ae2e2c2b38e991aa6114a37276`                                    |
| Inspection native en lecture seule                                                   | Version Projects et aides `thread`/`herdr worktree create` disponibles ; aucune création/reprise native rejouée                                             |

Les liens user existants Claude/Codex des skills `handoff` et `herdr-issue-worktree`,
et Cursor de `memory-governance`, pointent vers le checkout canonique.
Le lien Cursor déclaré de `herdr-issue-worktree` est absent avant ce changement :
sa projection temporaire fonctionne, mais son déploiement user actuel reste
non conforme. Aucun lien user n'a été modifié. Les trois adaptateurs projet
résolvent vers `../.agents/skills`.

Un contrôle CSpell supplémentaire des deux README a signalé 16 mots déjà présents
avant le diff, hors sélection de la tâche canonique ; ils ne sont pas corrigés ici.
Les ajouts contrôlés aux trois skills ne présentent pas d'erreur de spelling.
Les scénarios d'activation sont des cas raisonnés et une validation de format,
pas une preuve d'exécution multi-host. Aucun lancement supplémentaire d'agent,
dialogue trust, installation globale ou déploiement persistant depuis ce worktree.
Ubuntu est une cible des checks portables ; les plugins locaux sont déployés sur
macOS. Ubuntu, exécution native Linux, Windows/WSL et machines distantes ne sont
pas exercés ici. La CI distante reste requise avant merge.
