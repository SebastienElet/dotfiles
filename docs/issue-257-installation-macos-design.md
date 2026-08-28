# Design — Socle macOS et contrat du smoke

- **Issue** : [#257](https://github.com/SebastienElet/dotfiles/issues/257)
- **Statut** : décisions utilisateur approuvées le 28 août 2026, implémentées par #257

## Constat vérifié

L'ADR-001 fait du `Makefile` l'installateur unique et de `all` l'agrégat de référence. Dans
l'implémentation actuelle, `all` atteint `extra`, `terminal`, `work` et `utils`. Ces groupes
mélangent le socle de développement, des applications graphiques, des outils métier, des
applications payantes et des images Docker.

Le workflow macOS compense ce mélange. Sur `main`, il exécute `all` deux fois avec des exceptions
pour les applications payantes et l'absence de daemon Docker. Sur les pull requests,
`ci-smoke-targets` analyse les changements du `Makefile` et reconstruit une sélection de cibles.
L'ADR-023 qualifie cette sélection d'accélération consultative et conserve l'installation après
fusion comme autorité. Le smoke de `main` au commit `b894869` réussit sur `macos-latest` ; cette
preuve porte sur l'agrégat actuel et ses politiques de skip, pas encore sur le profil cible.

Cette architecture contredit la frontière recherchée par l'issue : le profil d'installation n'a
pas de source canonique et la CI reconstitue la sémantique de l'installateur. L'inventaire a aussi
révélé des outils qui ne sont plus utilisés. L'ADR-005 impose leur retrait plutôt que leur maintien
préventif.

Les choix ci-dessous consignent les décisions explicites du propriétaire pendant le brainstorming.
Ils ne remplacent ni le texte distant de #257 ni les ADR acceptées : ces autorités doivent être
alignées avant toute implémentation contradictoire.

## Décisions

### Deux axes indépendants

La portée cible possède quatre valeurs :

- `bootstrap` : prérequis nécessaire avant l'installation du profil minimal ;
- `minimal` : nécessaire au poste de développement de référence ;
- `optional` : utilisé, mais absent du poste minimal ;
- `retire` : inutilisé et destiné à être supprimé.

Le support cible en CI possède trois valeurs :

- `installable` : la CI peut installer le composant et en observer l'artefact ;
- `observable-without-install` : le runner fournit le prérequis, que la CI vérifie sans le gérer ;
- `skip-explicitly` : l'appelant choisit explicitement de ne pas installer le composant.

Seul le profil minimal recevra une garantie end-to-end en CI. #257 doit remplacer sa catégorie
exclusive `unsupported-in-ci` par cet axe indépendant et ajouter `retire` à la portée.

### Installateur hybride

Le `Makefile` restera l'orchestrateur des profils et des artefacts possédés par le dépôt. Il cessera
de porter l'inventaire détaillé des paquets Homebrew et Mac App Store.

```text
install.sh -> make minimal
                 |-- Brewfile
                 |-- installations non-Homebrew minimales
                 `-- configuration minimale

make optional ---|-- make minimal
                 |-- Brewfile.optional
                 `-- installations et configurations optionnelles
```

`Brewfile` deviendra la source canonique des paquets Homebrew et `mas` minimaux. [Homebrew
Bundle](https://github.com/Homebrew/brew/blob/main/docs/Manpage.md#bundle-subcommand) prend en charge
les formulae, casks et applications Mac App Store, puis expose `brew bundle check` comme probe.
`Brewfile.optional` portera les paquets optionnels. Un contrôle comportemental devra refuser les
doublons entre manifestes et toute répétition dans Make.

Le `Makefile` restera la source canonique pour les installations qui portent une autre logique :
Volta/npm, installateur Claude, images Docker, builds Rust locaux et symlinks. L'ADR qui remplacera
l'ADR-001 définit les frontières et référence ces sources sans recopier leur inventaire.

La formulation de #257 « depuis un seul document » doit être amendée en « depuis les sources
canoniques référencées par l'ADR » avant implémentation.

### Interface publique

- `install.sh` vérifie macOS, Apple Command Line Tools et Git, clone le dépôt, amorce Homebrew si
  nécessaire, puis délègue à `make minimal`.
- `make minimal` vérifie les prérequis externes, amorce Homebrew si nécessaire, puis installe et
  configure uniquement le profil minimal.
- `make optional` converge d'abord le profil minimal, puis installe les composants optionnels non
  exclus par les politiques explicites `SKIP_PAID_APPS=1` et `DOCKER_UNAVAILABLE_POLICY=allow-skip`.
- `all` n'est pas conservé comme alias. Les agrégats `terminal`, `work`, `utils`, `extra`, `ai` et
  `javascript` disparaissent après migration de leurs appelants.
- `tooling/upgrade` conserve la responsabilité des mises à niveau explicites du poste.

Apple Command Line Tools reste un prérequis manuel. Homebrew peut demander une élévation pendant son
bootstrap ; les profils ferment ensuite leur entrée standard et échouent si un cask ou une autre
dépendance exige une interaction.

## Inventaire cible approuvé

L'unité classée est le composant installable visible par l'utilisateur. Ses artefacts possédés —
thèmes bat, `fzf.fish`, TPM, configurations et intégrations d'agents — héritent de sa portée et ne
constituent pas une seconde entrée.

### Bootstrap

| Composant                       | CI                           | Justification                                                   |
| ------------------------------- | ---------------------------- | --------------------------------------------------------------- |
| Apple Command Line Tools et Git | `observable-without-install` | Fournissent Git et Make avant le clonage.                       |
| `curl`                          | `observable-without-install` | Exécute le point d'entrée publié et les téléchargements bornés. |
| Homebrew                        | `observable-without-install` | Gestionnaire nécessaire avant les deux `Brewfile`.              |

### Minimal

Tous ces composants devront être `installable` sur le runner macOS de référence avant que le profil
soit garanti.

| Domaine              | Composants                                                                                                                          |
| -------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| Terminal             | bat, bottom, broot, eza, fd, Fish, Fisher, fzf, git-delta, GNU sed, jq, lazygit, mtr, procs, ripgrep, Starship, tmux, tokei, zoxide |
| Éditeur et interface | Arc, JetBrains Mono, Neovim, WezTerm                                                                                                |
| Runtimes             | Bun, Node.js, OrbStack, pnpm, Rust, Volta                                                                                           |
| Agents et outillage  | `agent-handoff` avec sa cible Make, Arnes, Claude Code, CodeGraph, Codex, Hunspell et ses dictionnaires                             |

### Optionnel

| Domaine                | Composants                                                                                                      |
| ---------------------- | --------------------------------------------------------------------------------------------------------------- |
| Développement          | AWS CLI, bkt, CSpell, Doppler, GitHub CLI, GnuPG, k9s, lazydocker, Linear CLI, mosh, PostgreSQL, uv, Vale       |
| Agents et web          | ChatGPT, Claude, CloakBrowser, CodexBar, Cursor CLI, llmfit, Scrapling                                          |
| Applications           | CleanShot X, DaisyDisk, Google Chrome, Handy, LanguageTool, Rectangle Pro, Things 3 et son wrapper, Vibe Island |
| Support d'installation | `mas`                                                                                                           |

DaisyDisk et Things 3 devront échouer si l'artefact promis manque. `SKIP_PAID_APPS=1` les exclura
sans tentative. CloakBrowser et Scrapling exigeront un daemon Docker ; la migration retirera
`allow-skip` comme valeur par défaut et le conservera comme opt-out explicite.

DaisyDisk, Things 3, CloakBrowser et Scrapling sont `skip-explicitly`; tous les autres composants
optionnels sont `installable`, sans être exercés end-to-end. La portée `retire` sort du système
cible avant l'attribution du support CI.

Pour Doppler, GitHub, Linear, Bitbucket, Claude et Codex, le contrat cible s'arrête au binaire local.
Les tests ferment stdin et vérifient qu'aucune commande de connexion n'est lancée ; l'accès distant
reste hors périmètre.

### À retirer

| Type                    | Composants                                                                                                               |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| Paquets ou applications | 1Password, Flow, htop, Iosevka Nerd Font, jscpd, OpenSpec, Prettier, Renovate CLI, Skills CLI, TablePlus, Terraform, zsh |
| Outil maintenu          | `daily-routine`, son projet Rust, sa configuration, son déploiement, sa CI et sa documentation                           |
| Barrière locale         | `git-hooks`, le hook `pre-push`, son implémentation et ses tests                                                         |

Ces retraits sont les déclarations d'usage du propriétaire, pas une déduction depuis le dépôt. #257
doit supprimer sa contrainte de conservation de `daily-routine`, puis #177, #179 et #180 doivent
être alignées. Le hook disparaît sans remplacement local ; les mêmes validations TypeScript restent
exécutées par le workflow dédié.

## Découplages

- CodeGraph minimal configure Claude Code et Codex sans installer Cursor, qui reste optionnel.
- L'installation optionnelle de Cursor ajoute sa propre intégration CodeGraph.
- OrbStack minimal n'installe pas lazydocker.
- WezTerm minimal n'installe pas Iosevka Nerd Font.
- Doppler et GnuPG restent ensemble dans le profil optionnel, conformément à la
  [procédure officielle](https://docs.doppler.com/docs/install-cli) de vérification de signature.

## Convergence et erreurs

Un profil commencera par un probe silencieux en lecture. Le chemin convergé utilisera
`brew bundle check --quiet --no-upgrade`; l'installation ne sera lancée que si un paquet manque.
Le profil ne demande aucune mise à niveau globale, mais Homebrew peut mettre à niveau une dépendance
nécessaire à l'installation manquante.

Le propriétaire a explicitement décidé que chaque passage revalide les destinations possédées par
le dépôt. Cette décision cible contredit l'ADR-003 actuelle, qui devra être remplacée avant la
bascule ; elle ne constitue pas une exception locale anticipée.

Les futures issues doivent prouver ces règles par des tests de chemins contradictoires :

- une destination absente peut être créée ;
- une destination correcte reste inchangée ;
- une destination divergente, un lien symbolique pendant ou un type inattendu est préservé et
  provoque un échec explicite ;
- un installateur qui retourne `0` sans produire l'artefact promis provoque un échec ;
- une dépendance, un manifeste ou un probe absent, illisible ou invalide provoque un échec ;
- un tap Homebrew qui ne peut pas être utilisé ou approuvé provoque un échec ;
- aucun fallback ne transforme une preuve indisponible en succès.

Le préfixe Make `@` peut masquer l'écho d'un probe en lecture, jamais celui d'une mutation. Le
silence résulte d'un chemin convergé qui n'exécute que ces probes et ne trouve aucun travail.

## Contrat du smoke macOS

Le workflow cible aura un seul job sur `macos-latest`, sur chaque pull request et push vers `main`,
sans matrice ni sélection calculée depuis le diff. Il invoquera l'oracle public
`make smoke-minimal` sans énumérer les paquets ni les artefacts.

1. Vérifier les prérequis `bootstrap` fournis par le runner.
2. Exécuter `make minimal` une première fois.
3. Vérifier la satisfaction du `Brewfile`, les exécutables non-Homebrew et les destinations
   possédées par le dépôt.
4. Relever le type, la cible ou le contenu des artefacts possédés par le dépôt.
5. Exécuter `make minimal` une seconde fois en capturant `stdout` et `stderr`.
6. Exiger un statut `0`, deux flux vides, les mêmes postconditions et des artefacts identiques au
   relevé précédent.

Cette preuve cible est limitée aux artefacts relevés sur le runner exercé. Elle ne garantit ni les
écritures internes de Homebrew, ni les composants optionnels, ni une session authentifiée, ni le
démarrage du daemon OrbStack, ni Linux. Un échec de vérification fait échouer le job.

## Séquencement des issues d'implémentation

Avant publication de ces issues, #257 doit intégrer les décisions de taxonomie, de source canonique
et de retrait de `daily-routine` consignées ici.

### 1. Retirer les composants de poste inutilisés

Supprimer les paquets, applications et configurations classés `retire`, hors hook et
`daily-routine`. Le résultat observable est l'absence de leurs installations et de leurs références
actives, sans changement des composants conservés.

### 2. Supprimer la barrière locale `pre-push`

Supprimer entièrement le déploiement, l'implémentation et les tests de la barrière. La CI reste la
seule barrière enforcing ; aucun hook local de remplacement n'est introduit.

### 3. Supprimer `daily-routine`

Supprimer l'outil, ses artefacts de déploiement et sa CI. Actualiser #177, #179 et #180 afin que
leurs contrats ne promettent plus de préserver un outil absent, sans modifier leurs résultats
métier.

### 4. Introduire les profils hybrides

Rendre les profils `minimal` et `optional` disponibles à côté du chemin historique. Les deux
`Brewfile` deviennent les sources canoniques de leurs paquets ; les installations non-Homebrew
restent orchestrées séparément. Les découplages définis ci-dessus sont observables sans changer
encore le point d'entrée public.

### 5. Basculer le point d'entrée public

Faire de `make minimal` la commande appelée par l'installation et la mise à jour du poste. Exposer
le profil optionnel séparément et mettre à jour la documentation publique. Remplacer l'ADR-001 et
aligner les ADR-002, ADR-003, ADR-004 et ADR-023 avant la bascule ; vérifier la portée des ADR-018
et ADR-039. Conserver le comportement d'échec explicite du bootstrap.

### 6. Simplifier le smoke

Exposer l'oracle public du contrat minimal et le faire exécuter par le workflow macOS, y compris la
preuve de silence et d'absence de mutation au second passage. Supprimer le sélecteur dynamique et
les agrégats historiques après vérification de l'absence d'appelant.

Chaque brouillon d'issue devra nommer son oracle et sa frontière de rollback avant publication. Les
trois premières réduisent le graphe ; la quatrième introduit le nouveau chemin sans bascule ; la
cinquième change les consommateurs ; la sixième retire la compatibilité sans appelant.

## Non-objectifs

- Authentifier un service distant pendant l'installation.
- Garantir les composants optionnels en CI.
- Contrôler les écritures internes de Homebrew au-delà de ses postconditions publiques.
- Ajouter un parseur du `Makefile`, un manifeste intermédiaire ou un nouveau framework de test.
