# Évaluations du harnais

## Usage

`harness:check` compose les contrôles déterministes, sans agent ni appel LLM.
Les [cas](../harness/evals/cases.json), fixtures et variantes sont des entrées
exécutées par Arnes, pas des comptes rendus. Rust et les dépendances Bun du dépôt
sont requis ; appliquer les [précautions de worktree](../README.md#contributing-from-a-worktree).

## Procédure

```sh
moon run harness:check
```

Ne pas substituer `moon check harness` : sa sélection peut inclure des tâches
opérationnelles qui installent des plugins. La CI exécute les validations d'evals
et d'evidence ; les autres contrôles ont leurs workflows dédiés.

Un essai live consomme du quota et demande une autorisation explicite. Utiliser
un Codex CLI compatible avec les options d'isolation du runner, une authentification
disponible et un identifiant de modèle exact. Vérifier d'abord son démarrage :

```sh
cargo run --quiet --locked --manifest-path tooling/arnes/Cargo.toml -- eval preflight
```

Après autorisation, créer un répertoire temporaire privé pour les rapports :

```sh
eval_directory=$(mktemp -d)
moon run harness:eval -- --model YOUR_EXACT_MODEL_ID --only repository-literal,repository-known-path --runs 3 --variant-file harness/evals/variants/no-op.md --report "$eval_directory/baseline.json"
moon run harness:eval -- --model YOUR_EXACT_MODEL_ID --only repository-literal,repository-known-path --runs 3 --report "$eval_directory/candidate.json"
moon run harness:compare -- "$eval_directory/baseline.json" "$eval_directory/candidate.json"
```

Examiner les résultats et restituer les observations dans la PR ou la conversation.
Ne pas versionner ces rapports ponctuels. Le répertoire `harness/evals/evidence/`
reste requis par le validateur ; aucun rapport n'est nécessaire pour réussir.

## Limites et vérification

Le preflight vérifie le démarrage du CLI, pas l'authentification ni l'accès au modèle.
Les réplicats utilisent des HOME et workspaces synthétiques isolés, avec uniquement
la section `Context Management` des instructions. Les shims `cat`, `rg` et `fd`
mesurent les observations prévues ; d'autres lectures peuvent produire des faux négatifs.
Ils ne sont pas protégés contre un agent qui les altère volontairement.

Un timeout ou une sortie inexploitable donne `INVALID`, jamais `PASS`. Un résultat
non `PASS` retourne un code non nul après publication du rapport ; un chemin de
rapport existant est refusé. La comparaison exige des contrôles compatibles et
reste descriptive : elle ne prouve pas un gain causal ou statistique.

`harness:validate-evals`, `harness:validate-evidence` et `arnes:test` vérifient les
contrats et le moteur avec des fixtures, sans quota LLM. Un contrôle déterministe
vert n'est pas une preuve de comportement live.
