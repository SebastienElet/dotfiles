# Profil de session Codex

## Usage

Pour un mandat qui impose un couple modèle/effort, lancer Codex avec un profil
explicite. Le lanceur requiert Bun, Codex et des hooks approuvés. Les autres
sessions conservent leur lancement habituel.

## Procédure

Depuis le checkout canonique :

```sh
moon run harness:codex-profile
codex-profile gpt-6.1-sol high
codex-profile gpt-6.1-sol high exec 'Votre mandat'
```

Le modèle et l’effort sont les deux premiers arguments ; les arguments suivants
sont transmis à Codex. Les overrides restent possibles et sont comparés au couple
demandé. Dans Herdr Projects ou un autre lanceur, utiliser cette commande pour le
mandat concerné.

Le lanceur conserve `TERM=dumb` et les accès aux métadonnées Git du worktree
sélectionné par `--cd` ou `-C`, comme le wrapper Fish habituel.

Approuver les hooks par le mécanisme natif de Codex avant de compter sur leur
observation. Le lanceur ne contourne pas leur confiance et ne modifie aucun fichier
de configuration. Les hooks des autres couches restent chargés.

Examiner les diagnostics sur stderr, également disponibles comme messages de hook
dans l’interface native :

- `conforme` : modèle actif et effort du tour observés, identiques au profil ;
- `AVERTISSEMENT — différent` : au moins une valeur observée diffère ;
- `inconnu` : aucune différence établie et une observation manque.

Le démarrage reste inconnu pour l’effort. L’observation est renouvelée avant les
outils et à la fin du tour, y compris après une reprise ou un changement volontaire.
Un sous-agent identifié conserve son profil spécialisé : il est signalé hors
comparaison avec le profil racine. Pour un mandat spécialisé lancé séparément,
fournir son propre couple au lanceur.

## Limites et vérification

Le modèle vient du hook natif ; l’effort vient du dernier `turn_context` du même
tour et du même modèle dans le journal natif. Ce sont les valeurs actives du client,
pas une attestation du modèle servi ou d’un routage interne au fournisseur.
Le [format du transcript n’est pas une interface stable](https://learn.chatgpt.com/docs/hooks#common-input-fields).
Un journal absent, éphémère, illisible, incomplet ou sans contexte récent reste
inconnu ; aucune valeur n’est déduite du manifeste ou des arguments.

La lecture du journal et de l’entrée du hook est bornée à 1 Mio chacune. Les
diagnostics ne contiennent que le couple demandé, le couple observé et leur état.
Le relais Unix temporaire rend les messages visibles même avec `codex exec --json`,
qui omet les messages de hook. Il ne crée aucun historique de mesure ; son socket
est supprimé à la sortie normale du lanceur. Un arrêt non interceptable peut
laisser son répertoire temporaire vide de données de conversation.

Des hooks désactivés, non approuvés ou indisponibles ne produisent aucune preuve :
le diagnostic initial reste inconnu. Cette observation est un avertissement, sans
blocage ni rétablissement automatique des valeurs.
Un échec du relais fait échouer le hook avec un diagnostic inconnu ; chaque message
porte sur son observation, sans attester les événements ultérieurs.

`bun test tooling/codex-profile.test.ts` exerce les états et les chemins d’échec
par le point d’entrée public. `moon run tooling:deployment-test` exerce le lien,
le rejeu et les collisions dans des HOME temporaires sur macOS et Ubuntu.
