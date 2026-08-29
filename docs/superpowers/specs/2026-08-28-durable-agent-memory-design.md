# Mémoire durable locale pour agents

## Statut et portée

Ce document remplace le design candidate-only du 27 août 2026 pour la suite de la PR 249. Le
contrat candidate-only reste le comportement observé au commit `4ca15f5509f9509457c4d4b09027441dd65379b7`,
mais il ne satisfait pas la promesse de mémoire durable : il ne persiste, n'indexe, ne retrouve et
n'injecte aucune entrée.

La PR 249 doit livrer le système complet décrit ici. Le périmètre couvre Codex, Claude Code et
Cursor sur la machine locale. Il ne couvre ni synchronisation entre machines, ni stockage dans Git,
ni recherche sémantique, ni édition de l'état généré propre à un agent.

## Résultat attendu

Une connaissance utile détectée par un agent peut être proposée à l'utilisateur. Après une demande
explicite de mémorisation ou l'acceptation d'une proposition, elle est admise, persistée localement,
indexée et retrouvable dans les sessions suivantes. Au début d'une tâche, chaque agent recherche
les entrées pertinentes, contrôle la validité de leur preuve ou d'un cache récent, puis annonce les
entrées effectivement injectées avec leur source et leur fraîcheur.

Le système est utilisable seulement si une évaluation en processus frais prouve ce flux de bout en
bout pour chacun des trois agents. Une skill ou une instruction déclarative sans écriture et
retrieval observés ne suffit pas.

## Décisions de conception

- Les YAML locaux sont la source canonique et humaine des mémoires.
- Un index local dérivé borne le coût de recherche et le contexte injecté.
- Le store est partagé par Codex, Claude Code et Cursor sur une seule machine.
- Le scope projet est le défaut ; le scope utilisateur exige une admission explicite.
- La recherche est lexicale et déterministe, enrichie par plusieurs termes d'accès et alias.
- Chaque entrée possède une preuve et un oracle capable de la maintenir, la clore ou l'invalider.
- Un verdict d'oracle `valid` peut être réutilisé pendant 48 heures.
- Aucune donnée mémoire n'est versionnée. Seule l'implémentation du système appartient au dépôt.
- Une demande explicite écrit après admission ; une détection implicite propose sans écrire.

## Frontières des outils

Trois applications locales restent séparées par responsabilité :

- `tooling/agent-memory/` est un package Cargo autonome. Son binaire `agent-memory` possède seul le
  domaine mémoire : schéma, admission, identité projet, sources, stockage, index, oracles, cache,
  retrieval, transitions et adaptation des entrées et sorties runtime propres à chaque agent.
- `tooling/agent-handoff/` est un second package Cargo autonome. Il remplace l'exécutable Bun
  existant sans changer son contrat applicatif observable : mêmes octets stdin déjà reçus, mêmes
  variables d'environnement interprétées, mêmes données écrites sur stdout et stderr, et mêmes
  codes de sortie lorsque stdout est accessible en écriture.
- Arnes configure, valide et mesure les hooks et leurs exécutables. Il ne contient aucun domaine
  mémoire ou handoff, ne lit et n'écrit aucun de leurs états, et ne dépend d'aucun de leurs crates.

Les deux nouveaux packages possèdent chacun leur `Cargo.toml`, leur `Cargo.lock`, leurs tests et leur
binaire. Ils ne forment pas un workspace Cargo, ne partagent aucun crate interne et ne dépendent pas
l'un de l'autre. Une duplication locale est préférable à une abstraction commune sans invariant
métier commun établi.

Pour `agent-handoff`, la parité byte-for-byte commence après réception des octets stdin par le
processus et suppose une sortie stdout accessible en écriture. Les échecs de lecture stdin ou
d'écriture stdout au niveau du système d'exploitation sont hors de cette frontière : le runtime
Rust les normalise en `unexpected failure`, code 3, au lieu de reproduire les diagnostics natifs
instables du launcher ou du stream Bun. Toutes les erreurs applicatives après la frontière d'entrée
restent soumises à la parité exacte.

Le `Makefile` construit et déploie séparément `agent-memory` et `agent-handoff` sous
`~/.local/bin/`. Arnes installe et contrôle seulement les commandes absolues qui les invoquent dans
la configuration des agents. La réussite du build ou du diagnostic Arnes ne prouve donc aucun
comportement runtime de mémoire ou de handoff.

## Stockage local

Le store réside sous `~/.local/share/agent-memory/`, hors de tout dépôt Git. Le répertoire porte des
permissions `0700` et les fichiers `0600`. Il sépare les entrées de scope utilisateur et celles de
scope projet. Deux worktrees du même dépôt résolvent la même identité projet ; une identité absente
ou ambiguë bloque l'admission au lieu de créer un scope implicite différent.

Un fichier YAML autonome représente une mémoire. Son identifiant est stable après la première
admission. Les fichiers restent consultables directement, mais toute mutation supportée passe par
la frontière de stockage afin de préserver validation, atomicité, déduplication et indexation.

Les données peuvent contenir des invariants confidentiels de projet ou des éléments personnels
utiles. Les credentials, secrets, prompts privés complets et transcriptions brutes sont refusés. Le
store complet ne quitte jamais la machine. L'utilisateur accepte que les seules entrées retenues
pour une tâche soient transmises dans le contexte de l'agent concerné.

## Modèle d'une entrée

Chaque fichier respecte un schéma fermé et versionné :

```yaml
schema_version: 1
id: mem_<identifiant-stable>
kind: invariant
status: active
statement: <énoncé autonome et concis>
scope:
  type: project
  key: <identité stable commune aux worktrees>
retrieval_terms:
  - <terme canonique>
  - <alias ou traduction>
proof:
  summary: <ce qui établit ou justifie l'énoncé>
  sources:
    - kind: git-file | local-file | official-url | user-decision
      locator: <forme propre au kind>
      fingerprint: sha256:<64 hex>
  established_at: <date ISO et environnement>
oracle:
  automated:
    kind: source-fingerprint
    expected: all-proof-sources-unchanged
  human_fallback:
    question: <question>
    valid_when: <réponse observable>
  outcomes:
    valid: <condition observable>
    invalidated: <condition observable contraire>
created_at: <date ISO>
```

Les valeurs de `kind` sont fermées : `goal`, `decision`, `evidence`, `invariant`, `unknown` et
`assumption`. `proof` contient au moins une source. Chaque source emploie l'une des unions fermées
`git-file`, `local-file`, `official-url` ou `user-decision`, avec un `locator` propre à son type et
une empreinte `sha256:<64 hex>`. `oracle.automated` est obligatoire, sauf si toutes les sources de
preuve sont `user-decision`; `human_fallback` reste obligatoire dans tous les cas. Aucun champ ne
peut contenir une commande shell. Aucune commande shell n'est persistée ou exécutée depuis un YAML.

Une entrée terminale ajoute obligatoirement un objet `transition` contenant le statut précédent,
le nouveau statut, la date, le verdict d'oracle et une raison concise. Cet objet est absent tant que
l'entrée reste `active`. Une transition ne détruit ni la preuve initiale ni l'historique nécessaire
pour comprendre pourquoi l'entrée a cessé d'être consommable.

```yaml
transition:
  from: active
  to: <statut terminal autorisé par kind>
  at: <RFC 3339 UTC>
  verdict: valid | invalid
  reason: <texte concis>
```

Les premières sources supportées sont les fichiers suivis par Git, les fichiers locaux, les pages
officielles accessibles par URL et les décisions explicites de l'utilisateur. Une source non
supportée est rejetée. Ajouter une nature de source exige son propre parseur, ses échecs explicites
et ses tests de fraîcheur.

## Sémantique des types et statuts

`status` est un scalaire à la racine. Son union est discriminée par `kind` afin qu'un état sans sens
ne soit pas représentable.

| `kind`       | Statuts persistés autorisés                      |
| ------------ | ------------------------------------------------ |
| `goal`       | `active`, `achieved`, `abandoned`, `invalidated` |
| `decision`   | `active`, `superseded`, `invalidated`            |
| `evidence`   | `active`, `invalidated`                          |
| `invariant`  | `active`, `invalidated`                          |
| `unknown`    | `active`, `resolved`, `invalidated`              |
| `assumption` | `active`, `confirmed`, `invalidated`             |

La preuve et l'oracle ont une signification propre à chaque type :

| Type         | Ce que la preuve établit                     | Ce que l'oracle contrôle                   |
| ------------ | -------------------------------------------- | ------------------------------------------ |
| `goal`       | un objectif explicitement adopté             | atteint, abandonné ou toujours actif       |
| `decision`   | la décision et l'autorité qui l'a prise      | supersession ou changement de portée       |
| `evidence`   | une observation et sa provenance             | présence ou reproductibilité de l'artefact |
| `invariant`  | une règle établie par une autorité           | autorité, portée et empreinte inchangées   |
| `unknown`    | un manque d'information réel et pertinent    | réponse désormais disponible               |
| `assumption` | une hypothèse nécessaire et sa justification | confirmation ou falsification              |

Les verdicts d'oracle `valid`, `invalid`, `unavailable` et `needs_confirmation` ne sont pas des
statuts persistés. Un résultat métier certain entraîne la transition terminale autorisée par le
type : par exemple `achieved`, `superseded`, `resolved` ou `confirmed`, et exige un verdict
`valid`. Une contradiction de la preuve avec verdict `invalid` produit `invalidated`. Une
indisponibilité ou une ambiguïté n'altère pas le fichier et interdit seulement son injection pour la
consommation courante.

L'admission retourne séparément `stored`, `duplicate`, `rejected` ou `conflict`. Le statut
`candidate` disparaît du flux persistant : une proposition implicite reste dans la conversation et
n'est pas écrite avant acceptation.

## Admission et proposition automatique

Une demande explicite telle que « mémorise ceci » déclenche l'admission puis écrit immédiatement si
tous les contrôles passent. Sans demande explicite, un agent propose une entrée seulement lorsqu'il
détecte une connaissance durable, coûteuse à redécouvrir, prouvable, pertinente pour une session
future et absente du store. La proposition montre au minimum le type, l'énoncé, le scope, les termes
d'accès, la preuve et l'oracle. Le refus ou l'absence de réponse ne produit aucune écriture.

Avant toute création, l'admission :

1. valide le schéma discriminé, le scope et les termes d'accès ;
2. refuse les contenus interdits et les sources non supportées ;
3. résout chaque source et en calcule l'empreinte ;
4. exécute l'oracle automatisé ou obtient la confirmation humaine requise ;
5. recherche un doublon dans le même scope ;
6. crée atomiquement le fichier puis met à jour l'index dérivé.

Une entrée identique existante retourne `duplicate` sans écriture. Une identité déjà occupée par un
contenu différent ou une mise à jour concurrente retourne `conflict` sans fusion silencieuse. Une
écriture interrompue ne doit rendre visible ni fichier partiel ni index pointant vers une entrée
absente.

## Indexation et coût de contexte

L'index est un artefact local dérivé et intégralement reconstruisible depuis les YAML. Chaque ligne
contient l'identifiant, le type, le statut, le scope, les termes d'accès, les tokens normalisés et
dédupliqués du statement, un résumé court et le chemin du fichier. Des diagnostics dérivés séparés
par scope contiennent seulement l'identifiant, le contrôle et l'effet; leur inventaire couvre aussi
les YAML omis. L'index n'est jamais la source d'autorité d'une entrée et n'est jamais injecté en
bloc dans le contexte d'un modèle.

Chaque recherche s'exécute localement contre l'index, puis charge seulement les fichiers retenus.
Le classement privilégie les correspondances dans `retrieval_terms`, puis dans `statement`, après
normalisation déterministe de la casse, des accents et des séparateurs. Il n'utilise ni embedding,
ni service, ni similarité sémantique implicite. Le résultat injecté est limité aux cinq meilleures
entrées au-dessus du seuil de pertinence ; le système signale combien d'autres résultats ont été
écartés par cette limite.

Un index absent, périmé ou corrompu est reconstruit atomiquement sans modifier les YAML. Un fichier
illisible, un schéma futur ou une combinaison type/statut invalide est omis avec un diagnostic
précis. Aucune réparation ou suppression automatique n'a lieu pendant le retrieval.

## Retrieval, injection et fraîcheur

Au début de chaque tâche, l'adapter de l'agent fournit la requête active et les identités de scope
au retrieval commun. Les scopes projet et utilisateur sont interrogés, sans permettre à un projet
d'accéder aux mémoires d'un autre. Un résultat sans correspondance n'injecte rien.

Pour chaque résultat pertinent, le système consulte le cache d'oracle. Un verdict `valid` vieux de
moins de 48 heures autorise la consommation. À expiration, l'oracle est rejoué. Une modification
locale observable de la source invalide immédiatement le cache, même avant 48 heures. Pour une
source distante, le compromis accepté permet une ancienneté maximale de 48 heures.

Un oracle automatisé concluant produit `valid` ou `invalid`. S'il est indisponible ou ne peut pas
conclure, le fallback humain explicite peut produire un verdict `valid` ; en l'absence de réponse,
l'entrée reste omise. Tout verdict `valid`, automatisé ou humain, suit la même fenêtre de cache de
48 heures. Le cache est dérivé, séparé des YAML et ne peut pas servir de preuve de sa propre
fraîcheur.

Chaque record de cache lie le verdict à un digest de l'oracle déclaratif et à un `proof_digest`
opaque calculé sur les sources ordonnées complètes `{kind, locator, fingerprint}`. Les locators ne
sont pas conservés en clair dans le cache. Un changement de locator, d'ordre ou d'empreinte produit
un miss; seules les empreintes locales sont recalculées avant un hit encore dans la fenêtre.

Les conclusions humaines terminales forment une union fermée compatible avec le type : objectif
atteint ou abandonné, décision supersédée, inconnue résolue, hypothèse confirmée. La validation
humaine d'une preuve est un verdict `valid` distinct et ne crée jamais à elle seule une transition
métier; `evidence` et `invariant` ne possèdent aucun terminal humain.

L'agent annonce chaque mémoire injectée avec son `kind`, son `statement`, la source de sa preuve et
l'âge du dernier verdict valide. Les résultats `unavailable`, `needs_confirmation` ou devenus
terminaux sont signalés séparément et ne sont pas appliqués. Au tour suivant, toute nouvelle
consommation repasse au minimum par le contrôle du cache et des invalidations locales.

## Intégration des trois agents

Le domaine d'admission, le store, l'index et les oracles sont exclusivement implémentés par
`agent-memory`. Codex, Claude Code et Cursor ne possèdent que l'adaptation minimale nécessaire pour
déclencher proposition, admission et retrieval selon leurs surfaces supportées au moment de
l'implémentation. Aucun adapter ne duplique le schéma, le classement, la politique de
confidentialité ou les transitions de statut.

Pour Codex et Claude Code, Arnes configure et valide le hook supporté afin qu'il exécute le binaire
absolu `~/.local/bin/agent-memory`. Le binaire interprète l'entrée du hook, exécute le retrieval et
produit la réponse attendue par l'agent. Pour Cursor, la règle et la skill déclenchent le même
binaire sans confier de logique mémoire à Arnes. Une défaillance ou une absence de `agent-memory`
est un échec explicite de l'adapter et ne déclenche aucun repli vers Arnes.

Le handoff suit la même séparation : Arnes configure et valide le hook qui lance
`~/.local/bin/agent-handoff`, tandis que `agent-handoff` possède seul son comportement runtime. Sa
migration de Bun vers Rust est terminée seulement si des tests de caractérisation exécutent les
deux implémentations avec les mêmes octets stdin déjà reçus, environnement et erreurs applicatives,
puis prouvent des sorties stdout/stderr et codes de sortie identiques avec stdout accessible en
écriture avant le retrait de l'implémentation Bun.

L'automatisation ne peut pas être affirmée à partir de la seule présence d'une skill ou d'une règle.
Pour chaque agent, une session fraîche doit montrer que le retrieval est déclenché avant qu'une
mémoire pertinente influence la réponse. Les versions observées avant rédaction sont Codex CLI
`0.150.1`, Claude Code `2.1.250` et Cursor `3.15.6` sur macOS ; ces versions sont une baseline, pas
une garantie de compatibilité future.

## Échecs et comportement sûr

Les chemins suivants échouent sans injection ni écriture implicite :

- preuve ou oracle manquant ;
- source absente, non supportée, privée au-delà du contrat ou impossible à vérifier ;
- scope absent, ambigu ou différent du projet actif ;
- terme d'accès vide ou entrée structurellement invalide ;
- statut incompatible avec le type ;
- secret, credential, prompt privé complet ou transcription brute détecté ;
- collision d'identité, conflit concurrent ou échec d'écriture atomique ;
- index impossible à reconstruire ;
- oracle expiré qui échoue, reste ambigu ou devient indisponible ;
- version de schéma inconnue.

Les diagnostics nomment l'entrée, le contrôle échoué et l'effet observé sans reproduire de contenu
sensible. Les entrées terminales restent consultables dans le store mais sont exclues du retrieval
normal. Une opération d'audit explicite peut les lister sans les réactiver.

## Preuve comportementale et critères d'acceptation

Les tests de domaine couvrent les six types, toutes les combinaisons type/statut, la validation des
preuves et oracles, les contenus interdits, la déduplication, les conflits et chaque transition de
cycle de vie. Les tests de stockage couvrent création atomique, interruption, permissions et
concurrence. Les tests d'index couvrent reconstruction, corruption, déterminisme, isolation des
scopes, worktrees et limite de cinq résultats.

Les tests de fraîcheur couvrent cache valide avant 48 heures, expiration à 48 heures, modification
locale pendant la fenêtre, source distante indisponible, contradiction de preuve, fallback humain
et absence de mise en cache d'un résultat non valide.

Chaque package Cargo est construit, formaté, linté et testé depuis son propre manifeste, sans
workspace racine. Les tests Arnes prouvent uniquement la configuration, la validation et la mesure
des commandes de hooks. Les tests `agent-memory` prouvent le domaine et ses protocoles runtime. Les
tests `agent-handoff` caractérisent puis conservent strictement le contrat de l'exécutable Bun
remplacé après réception des octets stdin et avec stdout accessible en écriture, y compris
environnement, stdout, stderr et codes de sortie. Les tests Rust conservent séparément la
normalisation fail-closed des échecs de stdin et stdout au niveau du système d'exploitation.

Chaque agent possède une évaluation de bout en bout en processus frais avec condition sans
déploiement puis avec déploiement. Le chemin observé est obligatoirement
`agent → adapter configuré → binaire agent-memory → adapter → agent` ; un appel direct du binaire,
un test Arnes ou la seule présence du hook ne constitue pas cette preuve. Plusieurs réplicats
distinguent :

1. proposition automatique d'une connaissance admissible sans écriture préalable ;
2. écriture après acceptation ou demande explicite ;
3. persistance effective dans le store local isolé du fixture ;
4. retrieval pertinent dans une session suivante ;
5. annonce de la preuve et de la fraîcheur avant application ;
6. absence d'injection pour une requête sans rapport ;
7. rejet d'un contenu interdit ;
8. omission sur oracle indisponible et transition sur preuve invalidée.

Les artefacts d'évaluation utilisent un store temporaire et ne lisent pas le store personnel réel.
Les résultats bruts privés restent hors Git ; un rapport versionné conserve seulement protocole,
environnement, versions, verdicts normalisés et limites. La preuve macOS ne vaut pas pour Linux ;
les composants portables sont testés séparément sur chaque cible supportée par le dépôt.

Aucune promesse de persistance, retrieval, injection automatique, isolation ou fraîcheur n'est
faite sans un oracle end-to-end vert qui couvre la capacité annoncée sur l'agent et l'environnement
nommés. Les 26 checks verts du contrat candidate-only ne couvrent pas ce nouveau comportement.

## Migration future vers SQLite

Les YAML restent l'autorité pendant cette phase d'audit. Les identifiants stables, le schéma fermé,
les scopes, les statuts, les preuves et les oracles définissent une frontière indépendante du
format de recherche. Si plusieurs audits montrent que concurrence, volume ou reconstruction de
l'index deviennent coûteux, une évolution pourra importer ces entrées dans SQLite sans modifier le
contrat observable d'admission et de consommation.

Cette possibilité ne justifie ni couche de compatibilité abstraite ni double écriture dans la PR 249. La décision de migration devra reposer sur des mesures produites par le système YAML livré.

## Alternatives écartées

- Le maintien du contrat candidate-only : aucune mémoire n'est disponible dans une session future.
- SQLite dès cette PR : moins lisible pour l'audit humain et non justifié avant mesure du processus.
- Un service MCP avec embeddings : dépendances, daemon, coût et surface de confidentialité sans
  besoin de recherche sémantique établi.
- L'intégration du domaine mémoire ou du runtime handoff dans Arnes : elle confond la gestion du
  harnais avec les capacités qu'il configure et empêche leur évolution et leur preuve indépendantes.
- Un workspace ou un crate partagé entre `agent-memory` et `agent-handoff` : aucun invariant métier
  commun ne justifie ce couplage dans cette PR.
- L'édition de `~/.codex/memories/` : état généré propre à Codex, non partagé avec Claude ou Cursor.
- Le stockage dans le dépôt : risque de versionner du contexte confidentiel et absence de scope
  utilisateur réellement local.
- La revalidation sans cache : coût inutile dans une même période de travail.
- Un TTL d'expiration de la mémoire : il confond durée de vie de l'entrée et fraîcheur de sa preuve.
