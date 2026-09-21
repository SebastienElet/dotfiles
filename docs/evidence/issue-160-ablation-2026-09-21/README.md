# Preuves du lot A039.a — #160

[Analyse et limites](../../issue-160-ablation.md). Ce dossier complète, sans les réécrire, les
[quatre runs de calibration](../issue-160-calibration-2026-09-21/README.md).

- `manifest.json` : sources, modèle/effort demandés, environnement, ordre réellement exécuté,
  delta A/B, placebo, empreintes originales/exportées et normalisations.
- `fixtures.json` : prompts et fichiers synthétiques exacts ; P1/N2 sont identiques à ceux de la
  calibration, P2/N1 ont été figés avant cette extension.
- `grading-plan.json`, `masking-limits.json`, `scores-blinded.json` : critères figés et scores
  publiés avant levée du masque A/B ; les identités placebo étaient connues du coordinateur.
- `input-hashes.json` : empreintes des 259 fichiers figés. Les snapshots complets, y compris les
  27 skills exposées à Codex, sont conservés dans l'archive locale.
- `preflight.json` : contrôles natifs de la fixture défectueuse, de la correction de référence et
  du cas déjà correct. Les journaux CI P2 sont des données synthétiques, jamais une exécution CI.
- `summary.json` : jointure avec les conditions, comparaison des 28 runs par cas et condition,
  consommation, observations auxiliaires, compteurs non applicables et limites. Les quatre
  premières lignes de preuve restent dans le dossier de calibration référencé par empreinte.
- `run-05` à `run-28` : chaque dossier contient `events.jsonl`, export normalisé du flux natif,
  et `artifacts.json` avec stderr, réponse finale, diff exact encodé en JSON, code/durée du
  processus et contrôles indépendants. Les tests non applicables restent `null`.

Le résumé contient des comptes descriptifs, pas un score causal universel. Les observations de
routage vers d'autres skills et les commandes échouées sont conservées sans attribution forcée
à la phrase évaluée. Un code 127 d'un outil appelé par l'agent n'est pas, à lui seul, un run
techniquement invalide ; les captures et les résultats fonctionnels sont examinés séparément.

Les originaux exacts résident dans le dossier local `issue160-extension-20260921` remis avec la
tâche, hors télémétrie Arnes. Les fichiers destinés au dépôt normalisent le répertoire synthétique,
le home et le nom de compte de l'opérateur, le nom de machine et les identifiants de tâche. Le
manifeste relie chaque export à l'original par SHA-256. Les copies d'authentification sont
supprimées après usage et absentes des archives ; les caches du CLI ne sont pas retenus.

Les chemins `/synthetic/…` sont des identifiants de restitution. Pour rejouer, reconstruire un
home et un dépôt neufs depuis les sources et fixtures indiquées, employer la commande native du
manifeste et vérifier les versions effectives. Chaque nouvelle exécution reçoit de nouveaux IDs
et fichiers ; elle ne remplace aucun run existant. Aucun runner générique, schéma partagé ou
contrôle CI supplémentaire n'est introduit par ces artefacts.
