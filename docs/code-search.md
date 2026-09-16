# Recherche de code

Les trois agents utilisent la skill `code-search`. Les recherches exactes passent par `rg` et `fd`;
les recherches conceptuelles passent par `colgrep-search '<requête>'` depuis le checkout Git ciblé.

Le point d'entrée initialise ColGrep à la demande, prouve la racine canonique du checkout, valide
l'index et refuse les résultats extérieurs. En cas de refus, utiliser une recherche `rg`/`fd`
bornée. Aucun hook de création de worktree ne doit initialiser ColGrep.

## Contrôles Moon

`tooling:code-search-test` exécute les tests locaux du rappel et du point d'entrée avec
leurs fournisseurs de test, sur macOS et Ubuntu. Il dépend seulement de la préparation
Bun. Le scénario externe est exclu de cette commande, même si `COLGREP_INTEGRATION=1`
est hérité. Ces tests quittent la suite TypeScript générale ; l'agrégat `test` les conserve.
Les contrats de liens, de rejeu et de collision restent dans `tooling:deployment-test`.

`tooling:code-search-integration-test` conserve le scénario réel macOS entre un checkout
principal et deux worktrees divergents. Il prépare ColGrep et déploie le point d'entrée,
puis affiche la version du binaire utilisé par le scénario. L'appel sans argument exerce
le diagnostic et le code de refus du point d'entrée déployé avant les recherches.

Les entrées locales comprennent le rappel, les fournisseurs et les tests locaux. Les
sources du point d'entrée, son contrat et le skill sélectionnent les deux contrôles.
Le scénario externe, le déploiement et la source d'installation Moon sélectionnent
l'intégration. Le manifeste, le lockfile et les définitions Moon communes sont partagés.
Le Brewfile ne déclare plus ColGrep : le modifier ne sélectionne pas ces contrôles.

La sélection se vérifie avec les commandes natives, en fournissant un chemin modifié :

```sh
printf '%s\n' tooling/code-search-nudge.ts | moon ci --stdin --downstream none tooling:code-search-test tooling:code-search-integration-test
printf '%s\n' README.md | moon ci --stdin --downstream none tooling:code-search-test tooling:code-search-integration-test
moon action-graph tooling:code-search-integration-test --json
```

Sur un poste jetable macOS, une entrée comme `tooling/colgrep-search.ts` sélectionne aussi
l'intégration. En worktree de développement, inspecter le graphe puis utiliser
`--no-actions --upstream none` pour exercer un déploiement dans un HOME temporaire avec
les outils déjà disponibles ; l'installation complète reste réservée au runner CI.

Le cache de résultat reste désactivé : chaque sélection exécute le scénario, même avec
`--cache read-write`. Pour vérifier le rejeu, relancer cette commande sans `--force`, puis
changer le binaire désigné par `COLGREP_REAL_BIN` : une version illisible doit faire échouer
la tâche, sans restaurer le succès précédent. Ce paramètre choisit le même binaire pour
la version et les recherches. Une tâche non sélectionnée ne produit aucune nouvelle preuve
d'intégration ; le modèle distant et la formule Homebrew ne sont pas verrouillés.
