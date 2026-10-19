# Issue 580 — contexte appelant Herdr

## Correction locale

Les nouveaux agents Codex du lanceur d'issues utilisent `--no-daemon`. La préparation
transmet également cette consigne au lancement de l'agent d'implémentation. Cela conserve
le contexte de leur propre terminal au lieu de réutiliser celui d'un app-server partagé.

Le scénario natif décrit ci-dessous retrouve le panneau appelant sans identité saisie par
l'utilisateur, puis lance un Codex dans un worktree isolé. Aucune modification de Codex
ni configuration globale de panneau ou de socket n'est nécessaire.

Une session déjà attachée au daemon n'est pas réparée par ce changement. Son propriétaire
peut la reprendre depuis le terminal Herdr voulu avec :

```sh
codex --no-daemon resume <session-id>
```

Le correctif ne reprend aucune session automatiquement et ne modifie pas le wrapper Fish
général : `codex agents` et les connexions `--remote` conservent leur comportement natif.

## Diagnostic corrigé

L'[issue 580](https://github.com/SebastienElet/dotfiles/issues/580) rapporte un écart entre
les commandes du terminal utilisateur et celles des outils d'un agent.

La mesure initiale `printenv HERDR_ENV HERDR_PANE_ID HERDR_SOCKET_PATH HERDR_SESSION`
était trompeuse : sur le macOS exercé, `printenv` n'affiche que la première variable.
L'expérience `env TST580_FIRST=first TST580_SECOND=second /usr/bin/printenv TST580_FIRST TST580_SECOND`
n'affiche que `first`. Cela ne prouve pas l'absence des autres variables.

La lecture corrigée est :

```sh
env | rg '^HERDR_(ENV|PANE_ID|SOCKET_PATH|SESSION)='
```

Dans cette conversation, le panneau et le socket sont bien transmis. Le panneau hérité du
daemon est cependant périmé : `pane get` et les deux formes de `pane current` renvoient
`pane_not_found`, alors que `status --json` confirme un serveur compatible. Il ne s'agit
donc pas d'une indisponibilité générale de Herdr.

Les [sources Herdr 0.9.5](https://github.com/herdrdev/herdr/blob/v0.9.5/src/cli/pane.rs#L116)
confirment que `pane current` avec ou sans `--current` consomme le même panneau hérité.
La [sélection du socket](https://github.com/herdrdev/herdr/blob/v0.9.5/src/session.rs#L175)
dépend de la session explicitement demandée, sinon du socket hérité. Ni l'omission de
`--current`, ni le focus, ni un répertoire partagé ne rétablissent l'identité de l'appelant.

La [documentation officielle Codex](https://learn.chatgpt.com/docs/cli/reference) et l'aide
du binaire 0.163.0 exposent `--no-daemon` pour lancer le CLI interactif sans serveur partagé.
Cette option ne se combine pas avec `--remote` ou `codex agents`.

## Expérience native du 19 octobre 2026

Environnement : macOS ARM64, Fish 4.9.3, Codex 0.163.0, Herdr client/serveur 0.9.5,
protocole 22, Bun 1.4.3. Sources dotfiles au commit
`fbca9a31375bfb337298184aca2a589e38138fe3`, puis modifications locales de ce correctif.

Une session Herdr dédiée, un dépôt Git jetable et un `CODEX_HOME` isolé ont été utilisés.
La confiance dans les seuls chemins de fixture et l'accès réseau nécessaire au socket natif
sont configurés dans ce `CODEX_HOME`, sans modifier la configuration utilisateur.
Aucun dialogue de confiance n'a été accepté et aucun tracker réel n'a été modifié.

| Scénario                                                                      | Observation                                                                                                                                                                                           |
| ----------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Daemon lancé hors Herdr                                                       | Les outils d'un Codex qui rejoint ce daemon n'ont aucun contexte Herdr.                                                                                                                               |
| Daemon lancé avec le contexte synthétique `w1:p1` ; nouvel agent dans `w2:p3` | Ses outils héritent de `w1:p1` ; `pane current --current` renvoie cet autre panneau. Un succès de la commande ne prouve donc pas l'identité.                                                          |
| Même dépôt, Codex lancé par l'adapter corrigé avec `--no-daemon`              | Le Codex dans `w2:p2` retrouve `w2:p2`, avec le focus sur `w1:p1`.                                                                                                                                    |
| Plusieurs agents dans le même checkout, daemon toujours actif                 | Le Codex corrigé dans `w2:p4` reçoit son propre panneau et le socket de la session de test. La résolution renvoie `w2:p4`, pas l'agent voisin ni le panneau focalisé.                                 |
| Transmission depuis un agent corrigé                                          | Sans panneau ni PID fournis dans sa demande, l'agent dans `w2:p5` lit son contexte, inspecte son panneau et ses processus, crée le worktree et lance le Codex enfant dans `w4:p2` avec `--no-daemon`. |

Le worktree enfant porte le libellé exact `TST-581 — context inheritance fixture`,
la branche `tst581-child` et `is_linked_worktree=true`. L'enfant confirme son répertoire,
son Git top-level et sa branche, puis écrit exactement `TST-581 verified\n` dans
`child-proof.txt`. Des relectures indépendantes du fichier, de Git et de Herdr confirment
ces résultats. Le checkout source reste sur `main` au même HEAD ; les trois identifiants
de focus restent `w1`, `w1:t1`, `w1:p1`.

Le lancement du parent a renvoyé un timeout natif alors que son prompt était déjà en cours
d'exécution. Aucune relance ni seconde soumission n'a été effectuée. L'inspection ultérieure
confirme que le parent a terminé le scénario et que l'enfant a produit le fichier attendu.
Ce timeout n'est pas présenté comme un lancement acquitté avec succès.

## Vérification et limites

Le test unitaire du véritable adapter échoue avant correction sur l'absence de
`--no-daemon`, puis passe ; le cas Claude conserve ses arguments. Les contrôles TypeScript
du dépôt vérifient le code modifié. Les scénarios natifs vérifient la transmission réelle
du contexte, distinctement de ce test de construction des arguments.

La procédure conserve le refus d'un contexte absent, périmé ou ambigu. Elle vérifie le
processus appelant par sa propre ascendance et l'état natif, sans rechercher un autre
agent par répertoire ou par focus. Ces refus restent des instructions de workflow ;
aucun nouveau validateur ni contrôle d'identité exécutable n'a été ajouté.

Le chemin tracker complet, Claude en natif, les anciennes versions Codex, Linux,
Windows/WSL et les machines distantes n'ont pas été exercés. Les sources locales restent
à intégrer et déployer ; aucun redémarrage de serveur utilisateur ni reprise de travaux
Modelo n'a été effectué.
