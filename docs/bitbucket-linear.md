# Réconciliation Bitbucket Cloud → Linear

`tooling/bitbucket-linear-sync` inspecte les PR de tous les auteurs des dépôts
explicitement configurés. L'inspection est le mode par défaut ; `--apply` annonce
le plan complet avant la première mutation Linear. Bitbucket n'est jamais modifié.

```sh
tooling/bitbucket-linear-sync --config /chemin/reconciliation.json
tooling/bitbucket-linear-sync --config /chemin/reconciliation.json --apply --json
```

La configuration JSON ne contient aucun secret. Les identifiants ci-dessous sont
des exemples ; les UUID réels viennent de l'équipe et de ses états Linear.

```json
{
  "bktContext": "work",
  "linearWorkspace": "work",
  "repositories": [
    {
      "workspace": "acme",
      "repository": "app",
      "teamKey": "ENG",
      "teamId": "00000000-0000-4000-8000-000000000001",
      "completedStateId": "00000000-0000-4000-8000-000000000002"
    }
  ]
}
```

Chaque dépôt possède une équipe explicite. Les dépôts et clés d'équipe sont
uniques, et les déclarations répétées d'une même équipe doivent converger sur son
UUID et son choix d'état. Le contexte `bkt` doit viser `api.bitbucket.org` ; aucun
contexte ambiant n'est accepté. L'authentification appartient aux CLI existantes.
Les clés inconnues et configurations incohérentes sont refusées avant les lectures.

`completedStateId` est optionnel si l'équipe expose exactement un état de type
`completed`. Un choix explicite doit identifier un état de ce type dans l'équipe.
Une absence ou une ambiguïté bloque la clôture, tout en permettant de réparer les
pièces jointes certaines.

## Décisions et reprise

La commande lit les PR ouvertes, fusionnées et déclinées, leurs auteurs et leurs
dépôts source/destination. Elle suit chaque lien `next` Bitbucket et chaque curseur
Linear, vérifie l'identité du dépôt, l'URL canonique, les champs requis et les
résultats de mutation. Un lien de pagination qui change de dépôt ou d'origine,
une boucle, un curseur manquant, une réponse malformée ou une erreur GraphQL
partielle rend l'inventaire concerné incomplet. Les métadonnées de pagination
présentes sont validées ; un total annoncé qui contredit le nombre de PR
collectées interdit de déclarer l'inventaire complet.

La corrélation cherche toutes les clés d'équipe configurées dans le titre, la
branche source et la description, avec frontières lexicales et déduplication sans
sensibilité à la casse. Un seul identifiant distinct autorise la résolution dans
l'équipe du dépôt. Aucune mutation n'est faite pour une PR sans identifiant,
ambiguë ou associée à une autre équipe. Les PR ambiguës bloquent aussi la clôture
des identifiants qu'elles mentionnent, même lorsqu'une autre PR les résout.

Les pièces jointes certaines sont créées ou mises à jour avec l'URL Bitbucket
canonique, le titre et l'état courant. Une pièce jointe déjà conforme ne produit
aucune écriture ; des doublons existants bloquent les actions de l'issue.
L'idempotence de création/mise à jour utilise le contrat fournisseur
`attachmentCreate(issueId, url)`.

Une issue active devient éligible à la clôture si son instantané est complet,
contient au moins une PR fusionnée et aucune ouverte, et si toutes ses pièces
jointes ont été vérifiées. Une issue annulée, terminée ou archivée ne peut pas être
clôturée par cette commande. Une issue terminée avec une PR ouverte est signalée,
sans changement d'état. L'état et l'équipe sont relus avant la clôture ; un
changement depuis le plan bloque cette écriture. Cette relecture doit également
retrouver exactement une pièce jointe conforme par PR ; une suppression, un
doublon ou un titre/état divergent bloque la clôture, sans rejouer une réparation
qui ne figurait pas dans le plan annoncé.

Les mutations suivent l'ordre identifiant d'issue puis URL. Un échec de pièce
jointe interdit la clôture de son issue ; les autres actions indépendantes
continuent. Une panne d'inventaire bloque toutes les clôtures de l'équipe
concernée, y compris dans ses autres dépôts. Chaque invocation relit les deux
fournisseurs et recalcule son plan : aucune décision précédente n'est rejouée.

La décision repose sur un instantané réussi. Il n'existe aucune transaction entre
Bitbucket et Linear ni condition atomique protégeant une lecture contre tous les
changements concurrents. Une relance répare les dérives suivantes. Cette commande
ne lit pas les cases de preuve des issues et ne remplace pas le workflow agent
`linear-sync`.

## Sorties et erreurs

`--json` produit deux objets JSON sur deux lignes : `phase: "plan"`, puis
`phase: "result"`. Les statuts sont `unchanged`, `linked`, `updated`, `completed`,
`ambiguous`, `ignored` et `failed`. Le plan conserve les identités fournisseur et
les états observés utiles au diagnostic. Les descriptions des PR, les secrets et
les sorties brutes d'erreur des CLI ne sont jamais imprimés.

Le code de sortie est `0` pour un instantané complet et des actions réussies,
`1` pour un échec fournisseur, un inventaire incomplet, une ambiguïté ou un échec
de mutation, et `2` pour un usage ou une configuration invalide avant mutation.
Les commandes fournisseur sont séquentielles et bornées à trente secondes,
sans relance automatique. Après un timeout de mutation, le résultat est inconnu ;
la prochaine invocation relit l'état avant de recalculer les actions.

## Transports et portée des preuves

Qualification du 30 septembre 2026 sur macOS arm64 : aide locale de `bkt 0.32.1`
et `linear 2.6.0`, introspection GraphQL authentifiée des entrées
`AttachmentCreateInput` et `IssueUpdateInput`, sans lecture d'issue métier ni
mutation. Une lecture Bitbucket authentifiée a rencontré un timeout du keyring ;
ce résultat ne démontre ni absence de credentials ni fonctionnement E2E.

Sources officielles consultées à cette date :

- [Liste et états des PR Bitbucket](https://developer.atlassian.com/cloud/bitbucket/rest/api-group-pullrequests/).
- [Pagination Bitbucket et liens opaques](https://developer.atlassian.com/cloud/bitbucket/rest/intro/#pagination).
- [Pagination Linear](https://linear.app/developers/pagination).
- [Erreurs partielles GraphQL et transitions explicites](https://linear.app/developers/graphql).
- [Idempotence issue–URL des pièces jointes](https://linear.app/developers/attachments).

Le code TypeScript vise macOS et Ubuntu avec Bun et les deux CLI installées et
authentifiées. Les tests automatisés utilisent des fournisseurs en mémoire et
des réponses de transport synthétiques ; ils couvrent les décisions et refus
possédés, sans certifier les fournisseurs. Les checks Ubuntu relèvent de la CI.
Aucun environnement de test fournisseur n'est configuré implicitement.

La garantie opérationnelle reste non établie tant qu'un environnement hors
production explicitement désigné n'a pas permis de vérifier une collecte
authentifiée sur plusieurs pages et auteurs, l'idempotence réelle, le rejeu après
échec partiel et une clôture réelle avec relecture. Aucun essai de mutation réelle
n'est exécuté par la suite de tests ordinaire.
