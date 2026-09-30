# #160 — Observation marginale d’A020

Le lot du 30 septembre 2026 ne permet pas de décider du retrait d’A020 : aucun des six
appels ne satisfait son oracle. Il conserve cinq `FAIL` et un `INVALID`, sans retry,
réécriture de doctrine, changement de runner ni décision sur #255.

## Conditions et provenance

A020 est le repère historique de la phrase actuelle : « A one-off lookup of a literal
you already know stays a plain `rg`. » La condition A conserve exactement la section
`Context Management` de `harness/AGENTS.md`. La [condition B](../harness/evals/variants/without-a020.md)
retire seulement cette phrase, avec son espace de séparation et son repli de ligne.
Le reste de la section et la skill `code-search` sont identiques. Cette skill porte
elle-même une consigne de recherche exacte : l’expérience mesure donc un effet marginal
en présence de cette redondance, pas l’utilité générale du comportement.

Source Git : `71122871071add839a6afb7289669214470dbf57`, avec modifications applicatives
Arnes non commitées en cours dans ce checkout. Ce n’est pas une attestation du binaire
publié à ce commit. Le binaire local `tooling/arnes/target/debug/arnes` a été copié et
vérifié sous `/tmp/dotfiles-issue-audit/a020/arnes` avant les six appels, puis conservé
sans recompilation ni remplacement. Son empreinte et celles des sources d’évaluation
sont restées identiques après les six appels.

Environnement observé : macOS 27.0.1 arm64, Codex CLI 0.159.2, modèle demandé
`gpt-6.1-sol`, effort `high`. Bun 1.4.0 sert aux contrôles documentaires ; l’évaluation
exécute Arnes Rust 0.1.0. Une répétition par cas et condition, timeout de 120 secondes,
aucun plafond de tokens. Chaque appel reçoit une fixture, un HOME et un CODEX_HOME
temporaires ; les commandes de l’agent ont le réseau désactivé. L’accès au provider
reste nécessaire au modèle.

Le preflight du binaire figé a démarré Codex isolé sans requête modèle. Les cinq appels
terminés ont ensuite fourni des compteurs natifs ; l’interaction live a donc été observée.
L’identité du modèle résolu, la disponibilité générale du provider et l’absence de toute
capacité native supplémentaire ne sont pas attestées.

## Corpus, oracles et commandes

Les trois [cas existants](../harness/evals/cases.json) et la fixture `code-search-v1`
ont été réutilisés sans changement : `structural-v1` exige une lecture instrumentée
de skill précédant la recherche conceptuelle ; `literal-v1` exige un `rg` exact sans
recherche conceptuelle ; `known-path-v1` exige la lecture instrumentée du fichier connu
sans exploration. Aucun nouveau corpus, scorer ou oracle n’a été ajouté.

Chaque cellule a exécuté cette commande, avec son ID et son nom de rapport :

```sh
/tmp/dotfiles-issue-audit/a020/arnes eval run --model gpt-6.1-sol --reasoning-effort high --only code-search-literal --runs 1 --timeout-seconds 120 --report /tmp/dotfiles-issue-audit/a020/a020-with-literal.json
```

Les cellules B ajoutent `--variant-file harness/evals/variants/without-a020.md`.
Ordre réel : A structural, A literal, B literal, A known-path, B known-path, B structural.
Tous les appels ont retourné le code 1 et publié leur rapport ; aucune cellule n’est restée
non exécutée. Aucun appel n’a été rejoué après l’échec ou le timeout.

## Résultats bruts

| Cas        | A, avec phrase                                                                | B, sans phrase                                                      | Durée A / B         | Observations instrumentées A / B                                  |
| ---------- | ----------------------------------------------------------------------------- | ------------------------------------------------------------------- | ------------------- | ----------------------------------------------------------------- |
| structural | [INVALID : timeout](/tmp/dotfiles-issue-audit/a020/a020-with-structural.json) | [FAIL](/tmp/dotfiles-issue-audit/a020/a020-without-structural.json) | 120,011 / 104,669 s | 2 / 1 `colgrep-search` réussis ; aucune lecture de skill observée |
| literal    | [FAIL](/tmp/dotfiles-issue-audit/a020/a020-with-literal.json)                 | [FAIL](/tmp/dotfiles-issue-audit/a020/a020-without-literal.json)    | 18,865 / 24,326 s   | aucune / aucune                                                   |
| known-path | [FAIL](/tmp/dotfiles-issue-audit/a020/a020-with-known-path.json)              | [FAIL](/tmp/dotfiles-issue-audit/a020/a020-without-known-path.json) | 20,872 / 31,967 s   | aucune / aucune                                                   |

Les quatre cellules literal/known-path ont des appels d’outils comptabilisés mais aucune
observation de shim. Cela ne distingue pas un comportement inadéquat d’une lecture ou
recherche non instrumentée. Le timeout conserve ses observations partielles ; ses compteurs
de tokens et d’appels d’outils restent `null`, jamais zéro.

Les trois commandes natives `eval compare`, A comme baseline et B comme candidate,
ont retourné 0 : [structural](/tmp/dotfiles-issue-audit/a020/a020-compare-structural.json),
[literal](/tmp/dotfiles-issue-audit/a020/a020-compare-literal.json),
[known-path](/tmp/dotfiles-issue-audit/a020/a020-compare-known-path.json). Chacune indique
`comparable: true`, `claim: descriptive-only` et `passRateDelta: 0` ; `INVALID` demeure
dans le dénominateur. Un delta nul entre résultats sans aucun PASS ne prouve ni no-op,
équivalence ni préservation du comportement. L’unique répétition n’établit aucun effet
statistique ou causal.

Sur les cinq appels avec compteurs : 261 392 tokens d’entrée, dont 218 112 en cache,
2 228 tokens de sortie et 15 appels d’outils. L’appel structural A est incomplet sur ces
mesures. Durée totale des six appels : 320,710 secondes. Aucun coût financier ou quota
total n’est déduit de ces compteurs partiels.

## Empreintes SHA-256

| Surface                                      | Empreinte                                                          |
| -------------------------------------------- | ------------------------------------------------------------------ |
| Binaire figé                                 | `fcf8c13f8b1801d77650e5bd042ac16eea1249ac92d97c527d2aa67347b7f632` |
| Section A exacte                             | `fb90185ef433b51daec33755b863c68d332e4739f0cef302cd159b119ed190d0` |
| Variante B exacte                            | `f6a181e002b2607869348e51c8a212df9812ee60e1ce581ea59357009227001e` |
| `harness/AGENTS.md` complet                  | `12f98ea76b73642ae7244249a7788787b1b13d64c99bed8945e6ad532d9af68d` |
| `code-search/SKILL.md` complet               | `7a51e11ace59fc89b7e40ab004c86d0d96fa75cac709be05153b5adea2b23039` |
| `cases.json` complet                         | `05d313c31b7c3f563358858bd6e38972ca113c20c69398897246662ab31debfe` |
| Fixture JSON complète                        | `6592045b8881cdb52a093ea89b8798ae232bab28bd77aa576f96a6f33015f64c` |
| Trigger queries JSON complet                 | `27d9a41c62ada0d73c867a37f2a84e1264e5a4bd64b20f47d290999330b87170` |
| `fixtureRevision` native, fixture et binaire | `f3646c1e81ce61b39278f7366eed3aa0c4c99b2af43a7687812104bdf5db0b7d` |

Chaque rapport conserve son prompt synthétique exact, son empreinte et le contrat versionné
de son oracle. Les six rapports natifs et les trois résultats de comparaison ont été relus : aucun chemin
personnel, credential, transcript brut ou contenu privé n’y figure. Le [protocole eval](../harness/evals/README.md)
conserve ces contrats synthétiques et observations réduites ; ce dossier n’est pas le store
de télémétrie v2, dont l’[ADR-043 accepté](adr/043-telemetrie-arnes-minimale-et-bornee.md)
exclut les contenus collectés. Aucun hook de télémétrie n’est installé par cette expérience.

Les octets natifs sont conservés hors dépôt dans les artefacts locaux de cette conversation,
dans le dossier `a020-evidence/`, avec un manifeste SHA-256. Le lien vers cet artefact
est fourni dans la livraison locale ; ces fichiers ne sont pas publiés avec la PR. La copie de travail sous `/tmp/dotfiles-issue-audit/a020/`
reste temporaire ; aucun upload public ni rétention Git des rapports bruts n’est effectué. Le README de `harness/evals/evidence/` autorise des rapports JSON
à plat, pas une archive. Deux rapports structural bruts ne passent pas Prettier ; les
résultats de comparaison ne sont pas des `Report` et échouent au validateur de rapports.
Ils ne sont donc ni reformattés, ni retenus dans Git, ni exemptés des contrôles. Seuls
ce document et la variante expérimentale sont ajoutés au dépôt.

## Contrôles et portée

Le binaire figé valide les trois cas et 22 contrats d’activation avant les appels ; les
six rapports bruts passent sa validation native explicite hors dépôt. La découverte
automatique `eval validate-evidence` ne vérifie pas ces fichiers externes. Les trois
comparaisons passent leurs commandes natives distinctes. Prettier et
`git diff --cached --check` vérifient seulement les deux nouveaux fichiers indexés.
Ces contrôles de données ne changent aucun résultat live.

Pour revalider les octets locaux existants sans relancer le modèle :

```sh
/tmp/dotfiles-issue-audit/a020/arnes eval validate-evidence /tmp/dotfiles-issue-audit/a020/a020-with-structural.json /tmp/dotfiles-issue-audit/a020/a020-with-literal.json /tmp/dotfiles-issue-audit/a020/a020-with-known-path.json /tmp/dotfiles-issue-audit/a020/a020-without-structural.json /tmp/dotfiles-issue-audit/a020/a020-without-literal.json /tmp/dotfiles-issue-audit/a020/a020-without-known-path.json
/tmp/dotfiles-issue-audit/a020/arnes eval compare /tmp/dotfiles-issue-audit/a020/a020-with-structural.json /tmp/dotfiles-issue-audit/a020/a020-without-structural.json
/tmp/dotfiles-issue-audit/a020/arnes eval compare /tmp/dotfiles-issue-audit/a020/a020-with-literal.json /tmp/dotfiles-issue-audit/a020/a020-without-literal.json
/tmp/dotfiles-issue-audit/a020/arnes eval compare /tmp/dotfiles-issue-audit/a020/a020-with-known-path.json /tmp/dotfiles-issue-audit/a020/a020-without-known-path.json
```

Seuls `Context Management` et `code-search` sont installés. USER, SOUL, maintenance,
autres skills, instructions déployées, plugins, hooks et MCP ne font pas partie du contexte
expérimental installé ; leurs comportements ne sont pas mesurés. Les shims ne prouvent
ni activation interne de skill, ni qualité de ColGrep, ni résistance à un agent qui les
contourne. Aucune inférence vers Claude, Cursor, Linux ou un autre modèle/effort. #160
reste ouverte ; A020 conserve sa formulation, avec un résultat marginal non concluant.
