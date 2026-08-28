# Validation de `design-claim-audit`

## Décision de réflexion

`propose — subagent-routing`.

Le résultat observable de la PR Bitbucket `septeo-immobilier/modelo-suite#645` est la présence de
cinq familles de garanties documentaires plus fortes que l'autorité, le mécanisme ou l'oracle
disponible. Elles concernent l'autorité d'un contexte, la complétude d'un ensemble distribué, une
validation relationnelle, une frontière atomique, des gates métier et une portée juridique.

La cause n'est pas établie : les constats proviennent d'une même PR et peuvent partager une prémisse
erronée. Le candidat est classé `harness-gap` uniquement parce qu'un audit depuis un contexte frais
peut plausiblement changer le résultat sans enseigner un contournement du produit.

## Candidat

- **Déclencheur** : création ou modification matérielle d'un ADR, design ou spécification de domaine
  qui revendique autorité, complétude, unicité, atomicité, clôture d'état, validation ou effet
  juridique.
- **Comportement attendu** : le contexte auteur délègue un audit en lecture seule à un agent frais,
  puis affaiblit ou retire toute garantie sans autorité canonique, mécanisme exact, portée explicite
  et oracle comportemental vert.
- **Portée** : skill user déployée sur Codex ; aucune publication, mutation de forge ni revue de PR
  ouverte. Claude Code et Cursor restent hors promotion sans trial propre.
- **Contre-exemple** : correction orthographique ou wrapping sans changement de sens.
- **Falsificateur** : une session conserve une garantie contradictoire, certifie elle-même son
  travail sans identité d'auditeur, ou invente une cible d'architecture sans décision approuvée.
- **Expiration** : le mainteneur du dépôt réévalue au 30 novembre 2026 ou dès que le journal manuel
  ci-dessous atteint dix activations réelles ; il retire la skill après deux échecs consignés, une
  régression de sécurité ou un veto utilisateur.

## Journal d'activation

Une activation ne compte que si sa date, son hôte, son résultat et un lien vers une preuve durable
sont ajoutés ici par le mainteneur. En l'absence de dix entrées vérifiables, seule l'échéance datée
déclenche la réévaluation.

| Date | Hôte | Résultat | Preuve |
| ---- | ---- | -------- | ------ |

## Environnement

- macOS, worktree `.worktrees/design-claim-audit-trial` ;
- `gpt-5.6-terra`, effort `high` ;
- `codex-cli 0.150.1`, processus `--ephemeral`, sandbox `read-only` et enfants isolés ;
- fixture synthétique sous `harness/skills/design-claim-audit/evals/fixture/` ;
- aucune donnée privée de `modelo-suite` copiée dans le dépôt.

## Trial comportemental

Le contrôle sans skill a été exécuté cinq fois sous trois pressions combinées : ticket produit
`Done`, pipeline verte et délai déjà consommé. Les cinq contextes frais ont détecté les
contradictions, mais aucun n'a délégué une seconde lecture indépendante. Ce contrôle confirme donc
l'efficacité d'un contexte frais et l'absence naturelle du routage, pas un manque de doctrine.

Les deux premières formulations du candidat ont échoué : le premier agent s'est considéré comme
son propre auditeur et a inventé des cibles ; le second s'est encore considéré comme son propre
auditeur. La formulation retenue exige un appel à l'outil de délégation, une identité de tâche et
interdit de créer une cible sans décision approuvée.

Une compression ultérieure a réintroduit le même défaut : au deuxième réplicat, l'agent a donné sa
propre identité comme auditeur. Le prédicat retenu exige donc une identité de tâche enfant différente
de celle de l'auteur, puis le compteur a été remis à zéro.

Une vérification après revue a encore échoué : l'agent chargé de la modification s'est déclaré
auditeur de son parent sans déléguer. La définition retenue identifie désormais la tâche qui modifie
le document comme auteur, même si un parent lui a confié tout le travail, puis remet à nouveau le
compteur à zéro.

Après cette remise à zéro, un premier essai a réussi : la tâche auteur
`/root/isolated_context_eval_2` a créé l'enfant isolé
`/root/isolated_context_eval_2/independent_claim_audit`, reçu son ledger complet, puis réconcilié les
neuf contradictions sans inventer de cible ni de dette. Ce succès unique ne remplace pas une série de
réplications.

L'essai suivant a échoué : l'auditeur a classé deux mécanismes absents comme `target` sans décision
approuvée, même si l'auteur ne les a ensuite pas conservés. L'ordre de décision des statuts exige
désormais une décision citée pour `target`, un ticket cité pour `debt`, et `contradicted` dans les
autres cas ; le compteur est remis à zéro.

Après trois succès consécutifs, une nouvelle tâche s'est encore déclarée auditeur de son parent et a
produit elle-même le ledger. La procédure exige désormais d'enregistrer d'abord la tâche courante
comme `author_task`, de créer `auditor_task` avant toute lecture des preuves, puis d'attendre son
ledger avant réconciliation ; le compteur est remis à zéro.

Le premier test isolé de cette procédure a correctement délégué et réconcilié le document, mais son
livrable a réduit le ledger à trois colonnes. Le contrat de sortie exige désormais, dans l'ordre, les
deux identités, les sept colonnes complètes, puis le document réconcilié ; le compteur est remis à
zéro.

La formulation initiale, structurée selon les conventions locales et limitée à 491 mots, a produit
cinq succès consécutifs. Chaque processus auteur a créé un enfant isolé avant de lire les preuves,
restitué les sept colonnes, retiré les garanties contradictoires et n'a inventé ni cible ni dette.

| Session Codex                          | Auditeur                 | Claims | Résultat |
| -------------------------------------- | ------------------------ | ------ | -------- |
| `01a04864-0599-7db0-b466-adc667bc6a45` | `/root/isolated_auditor` | 10     | succès   |
| `01a04864-07c2-7563-906b-a61496f3d43a` | `/root/isolated_auditor` | 9      | succès   |
| `01a04864-0663-7410-86af-e905a9f52126` | `/root/isolated_auditor` | 7      | succès   |
| `01a04866-4268-7120-ac43-1c76f7a3bb46` | `/root/isolated_auditor` | 7      | succès   |
| `01a04866-4159-7422-9016-cc52ae5eed67` | `/root/isolated_auditor` | 9      | succès   |

Une revue a ensuite établi que l'enfant générique pouvait réactiver le skill et héritait du sandbox
de l'auteur. Le candidat déploie désormais le rôle Codex `design_claim_auditor`, dont le fichier
TOML demande `sandbox_mode = "read-only"` par défaut, interdit la réactivation et interdit toute
nouvelle délégation. Ce mécanisme suit le [schéma officiel des agents Codex](https://developers.openai.com/codex/subagents.md#custom-agent-file-schema).

Codex n'a pas chargé ce rôle depuis un symlink project temporaire ; le même fichier TOML régulier a
été chargé. Un lancement parallèle sur quatre sessions a ensuite rendu le rôle indisponible dans une
session, qui a échoué fermée comme prévu. Le compteur a été remis à zéro et cinq lancements
séquentiels depuis un parent `workspace-write` ont tous utilisé le rôle dédié, rendu le ledger à sept
colonnes, retiré les sept garanties et n'ont modifié aucun fichier.

| Session auteur                         | Session `design_claim_auditor`         | Claims | Résultat |
| -------------------------------------- | -------------------------------------- | ------ | -------- |
| `01a04890-8839-7dc2-9f4b-af22f2dbebd4` | `01a04890-cd91-7281-b926-7df49a8a995a` | 7      | succès   |
| `01a04892-5de5-7e40-a677-1d7f23493946` | `01a04892-a6cc-75e0-a2f5-4f09512464f0` | 7      | succès   |
| `01a04893-cf73-7612-be1d-bade9bfa4980` | `01a04894-1a15-7f03-92d1-ceb161a3e261` | 7      | succès   |
| `01a04895-2cb9-7ab2-ac4f-98ab76cc36f8` | `01a04895-7fc3-74c0-b84e-c39cd260ab1f` | 7      | succès   |
| `01a04897-52b9-79c0-acb7-69339b259ae4` | `01a04897-9908-77f1-88cb-aebdc985ea50` | 7      | succès   |

Chaque sortie a été lue manuellement. Les sessions éphémères ne conservent pas leurs transcripts ni
leurs résultats bruts : cette table trace l'observation, mais ne constitue pas un test de régression
rejouable. Le contre-exemple éditorial antérieur n'a déclenché ni délégation ni ledger.

Le fichier `evals/trigger-queries.json` décrit trois activations et trois non-activations. Il ne
constitue pas une preuve d'exécution de ces scénarios sur un autre hôte.

L'exclusion des PR ouvertes dans le frontmatter a produit cinq non-activations sur le contre-exemple
avec `gpt-5.6-luna` : `01a04883-3cef-7a70-8a0f-005b4cac902d`,
`01a04883-3cf1-72e0-9c4b-4abe19d7f593`, `01a04883-3cfb-7381-9a66-b932fee2f1ba`,
`01a04883-3cfc-7601-bf87-353cd8787e9d` et `01a04883-3cf1-7db1-b33d-6760c673b27f`. Cette exclusion
ne route pas implicitement vers le skill manual-only `pr-verdict`.

## Limites

- Le fixture concentre les sources pertinentes dans six petits fichiers ; il ne mesure pas la
  découverte dans un monorepo réel.
- Les cinq réplications portent sur un seul modèle et un seul environnement.
- Une élévation de sandbox active sur le parent peut remplacer le défaut `read-only` du rôle ; le
  harnais ne revendique donc pas l'immuabilité comme garantie d'isolation.
- Le trial prouve le routage et le traitement de contradictions accessibles, pas que tout futur
  auditeur retrouvera chaque source nécessaire.
- La gate CSpell actuelle n'est pas qualifiée pour ce rapport français : la CI le formate mais ne le
  soumet pas à ce contrôle orthographique.
