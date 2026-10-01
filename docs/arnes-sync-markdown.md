# Synchronisation des instructions, prompts et commandes

Ces opérations synchronisent uniquement les projections déclarées dans le
manifeste. L'agent, la portée et la ressource sont explicites :

```sh
arnes sync instructions --agent claude --scope user
arnes sync instructions --agent claude --scope project
arnes sync instructions --agent codex --scope user
arnes sync prompts --agent claude --scope user
arnes sync prompts --agent cursor --scope project
arnes sync commands --agent claude --scope project
```

La commande annonce sa sélection et distingue les résultats appliqués, conformes,
refusés, échoués, vides et non pris en charge. Un résultat partiel ou refusé ne
constitue pas un succès. Aucune instruction, commande ou session d'agent n'est
exécutée pour faire cette synchronisation.

## Représentations prises en charge

| Ressource    | Agent et portée     | Publication                                          |
| ------------ | ------------------- | ---------------------------------------------------- |
| Instructions | Claude user         | Liens vers les sources du harnais                    |
| Instructions | Claude project      | Fichier contenant l'include de la source déclarée    |
| Instructions | Codex user          | Contenu assemblé suivant le renderer du Doctor       |
| Prompts      | Claude user/project | Fichier direct, contenu rendu ou lien vers la source |
| Prompts      | Cursor project      | Fichier direct ou contenu rendu déclaré              |
| Commandes    | Claude user/project | Même artefact que la projection du prompt référencé  |

Cursor instructions, Codex project instructions, Codex prompts, Cursor user
prompts et les commandes Cursor/Codex ne sont pas pris en charge. Les projections
de prompts Cursor par symlink restent `Unsupported` : leur contrat natif n'est pas
établi.

Les includes, leur frontière et leurs cycles sont contrôlés avec le résolveur
existant. Les projections de prompts utilisent exactement les représentations
`file`, `rendered` et, pour Claude, `symlink` reconnues par Doctor. Une projection
liée conserve le texte direct de la source ; elle n'assemble pas les includes.
Doctor vérifie la source, les includes et les variables, puis le type de
destination, sa frontière, sa cible canonique attendue et le contenu lu à travers
le lien. Les variables déclarées sont des noms de références : la synchronisation conserve `$NAME`, `${NAME}` et les
arguments natifs dans le texte ; elle n'invente aucune valeur à substituer.

Une commande utilise la projection Claude du prompt déclaré. Son nom doit
correspondre au chemin reconnu par le registre natif, et sa description doit déjà
correspondre au frontmatter de cette représentation. Une incompatibilité est
refusée ; aucune description n'est injectée dans une copie concurrente du prompt.
Les passages `prompts` puis `commands`, ou dans l'autre ordre, utilisent donc le
même artefact et le même propriétaire de publication.

## Propriété et rejeu

Une destination absente peut être créée. Un lien d'instructions Claude user ou de
prompt Claude user/project attendu reste inchangé, sans reçu ; tout autre lien,
fichier ou type de destination est conservé avec refus. Les instructions locales Claude project qui incluent déjà la source sont
conformes et restent intactes. Un fichier project préexistant sans cet include et
sans provenance n'est jamais modifié pour l'ajouter.

Pour les fichiers réguliers créés par ces opérations, un reçu adjacent porte le
propriétaire, la racine, la destination, le SHA-256 publié et l'identité du fichier :

```text
.claude/commands/review.md
.claude/commands/.review.md.arnes.json
```

Le reçu est écrit après publication et vérification de l'artefact. L'identité
vient du snapshot validé par l'écrivain atomique. Une mise à jour obsolète exige
que le reçu corresponde au fichier encore présent, à son contenu et à sa
déclaration. Un changement local, un remplacement du fichier, un reçu absent,
malformé ou incompatible interdisent cette mise à jour. Les sources canoniques ne
sont jamais adoptées comme destinations mutables.

Un fichier déjà conforme sans reçu est signalé conforme, sans création de reçu ni
adoption. Lorsque la source évolue ensuite, sa divergence ne donne aucun droit
implicite de le réécrire. Les prompts locaux ou possédés par des plugins restent
ainsi préservés même lorsqu'ils ressemblent au rendu attendu.

À état conforme, le rejeu ne réécrit ni le fichier ni son reçu. Les chemins
symboliques, liens physiques multiples et parents divergents ne constituent pas
des raccourcis d'adoption.

## Échecs et limites

Toute la sélection est préparée avant publication : capacités, sources, includes,
variables, collisions, descriptions et preuves de propriété. Une erreur de cette
préparation laisse les destinations sélectionnées intactes. La source est validée
et sa représentation est recalculée avant de publier chaque artefact ; un
changement observé arrête cette publication.

L'écrivain natif refuse les traversées de parents symboliques, vérifie les
snapshots et publie par fichier temporaire puis renommage. Un échec ne produit
pas de fichier tronqué. Il conserve les diagnostics observables sans imprimer les
contenus ou valeurs sensibles. Les fichiers de verrouillage et les répertoires
nécessaires appartiennent à cette mécanique de publication.

L'artefact et son reçu sont deux publications distinctes. Un échec du reçu après
publication de l'artefact est signalé comme échec ; l'absence de reçu n'est pas
transformée en preuve de propriété au prochain passage. Il n'existe aucune
transaction globale entre ressources, ni garantie d'isolation contre tous les
écrivains concurrents. L'identité enregistrée empêche qu'un remplacement observé
du fichier devienne une autorisation de mise à jour.

La conformité se vérifie avec les commandes existantes :

```sh
arnes doctor instructions --agent claude --scope user
arnes doctor prompts --agent claude --scope user
arnes doctor commands --agent claude --scope user
```

Les tests de cette livraison exercent la vraie CLI sur des répertoires temporaires,
les résultats du Doctor, le rejeu, les refus et les échecs de publication. Les
preuves locales sont produites sur macOS arm64 avec Rust/Cargo 1.98.1 ; Ubuntu est
la seconde cible et reste à vérifier en CI. Des fichiers conformes ne prouvent
pas qu'une session Claude, Cursor ou Codex les a effectivement chargés.

L'[expérience native de #158](https://github.com/SebastienElet/dotfiles/issues/158#issuecomment-5916221908)
avec Claude Code 2.1.280 a observé `/probe-user` dans `slash_commands` à l'initialisation avec une configuration isolée et un
lien user, contrairement au témoin absent. Elle n'a pas invoqué la commande ni
vérifié l'authentification ou le chargement complet des includes. Cette observation
ne garantit pas l'exécution d'une commande liée dans une session Claude ; la
prise en charge project est vérifiée ici par les oracles Arnes.
