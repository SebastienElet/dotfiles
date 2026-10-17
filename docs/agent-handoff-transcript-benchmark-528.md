# Mesure de la lecture des transcripts par `agent-handoff` (#528)

## Décision

Ne pas modifier le lecteur pour l'instant. Sur le Mac mesuré, le coût médian d'un
appel du binaire release est de 12 ms pour 5,1 Mo et de 35 ms pour 20,4 Mo. Un
échantillon local de 472 transcripts JSONL avait une médiane de 0,66 Mo, un
95e percentile de 4,54 Mo et un maximum de 21,05 Mo. Le coût observé reste
faible pour ce hook de fin de tour, y compris près du maximum observé. La
lecture complète et les références de lignes restent linéaires ; cette mesure
ne démontre pas que des sessions beaucoup plus longues seraient négligeables.

## Protocole

Mesure du 17 octobre 2026 sur macOS 27.0.1, `Darwin arm64`, `Mac16,6`, 64 Gio
de mémoire ; `rustc 1.98.1 (48a229cea 2026-09-01)`. Le binaire a été construit
avec `cargo build --release --locked` depuis le manifeste
`tooling/agent-handoff/Cargo.toml` du commit `51bcd83d`.

Chaque entrée contient un préfixe de 0, 60 000, 300 000 ou 1 200 000 lignes
JSON sans usage, puis les **mêmes 500 lignes** : 499 objets vides et un record
Codex à 42 tokens. Ce dernier reste sous le seuil de handoff. Le hook retourne
donc sans créer de sentinel. Les quatre suffixes ont le même SHA-256 :
`04ffe39c6eee880b07281e6127e553a4a834df183c8058668a3c0caab49c3a80`.
Le test mesure le processus complet, dont le lancement et la lecture de
l'événement sur stdin. Les tailles ci-dessous sont les octets de transcript
passés à `fs::read`, pas les octets physiques lus sur disque après cache.

| Lignes de préfixe | Octets du transcript | Médiane de 5 séries de 30 appels | Par appel | Pic RSS observé |
| ----------------: | -------------------: | -------------------------------: | --------: | --------------: |
|                 0 |                1 629 |                           0,11 s |    3,7 ms |      non mesuré |
|            60 000 |            1 021 629 |                           0,17 s |    5,7 ms |          4,2 Mo |
|           300 000 |            5 101 629 |                           0,35 s |     12 ms |         14,3 Mo |
|         1 200 000 |           20 401 629 |                           1,06 s |     35 ms |         44,0 Mo |

Les cinq séries du plus grand fichier ont pris 1,07, 1,06, 1,06, 1,07 et
1,06 s. Les valeurs sont des mesures sur ce poste, sans garantie sur Linux,
sur d'autres Mac ni sur un disque froid.

## Reproduction

Depuis la racine du dépôt, avec Cargo dans `PATH` :

```sh
(
  set -eu
  benchmark_dir=$(mktemp -d)
  trap 'rm -rf "$benchmark_dir"' EXIT
  CARGO_TARGET_DIR="$benchmark_dir/target" cargo build --manifest-path tooling/agent-handoff/Cargo.toml --release --locked
  binary="$benchmark_dir/target/release/agent-handoff"
  awk 'BEGIN { for (i=0; i<499; i++) print "{}"; print "{\"type\":\"event_msg\",\"payload\":{\"type\":\"token_count\",\"info\":{\"last_token_usage\":{\"input_tokens\":42},\"model_context_window\":100000}}}" }' > "$benchmark_dir/suffix.jsonl"
  for lines in 0 60000 300000 1200000; do
    awk -v n="$lines" 'BEGIN { for (i=0; i<n; i++) print "{\"type\":\"other\"}" }' > "$benchmark_dir/prefix.jsonl"
    cat "$benchmark_dir/prefix.jsonl" "$benchmark_dir/suffix.jsonl" > "$benchmark_dir/$lines.jsonl"
    printf '{"hook_event_name":"Stop","session_id":"benchmark-528","transcript_path":"%s/%s.jsonl"}\n' "$benchmark_dir" "$lines" > "$benchmark_dir/$lines.event"
    wc -c < "$benchmark_dir/$lines.jsonl"
    tail -n 500 "$benchmark_dir/$lines.jsonl" | shasum -a 256
    output=$(XDG_STATE_HOME="$benchmark_dir/state" HANDOFF_TOKEN_THRESHOLD=1000000 "$binary" < "$benchmark_dir/$lines.event")
    test -z "$output"
    for trial in 1 2 3 4 5; do
      XDG_STATE_HOME="$benchmark_dir/state" HANDOFF_TOKEN_THRESHOLD=1000000 /usr/bin/time -p /bin/sh -c 'i=0; while [ "$i" -lt 30 ]; do "$1" < "$2" > /dev/null || exit; i=$((i+1)); done' sh "$binary" "$benchmark_dir/$lines.event"
    done
    XDG_STATE_HOME="$benchmark_dir/state" HANDOFF_TOKEN_THRESHOLD=1000000 /usr/bin/time -l "$binary" < "$benchmark_dir/$lines.event" > /dev/null
    test ! -e "$benchmark_dir/state/dotfiles/handoff/benchmark-528"
  done
)
```

Le pic RSS utilise l'option macOS `time -l`. L'échantillon de tailles réelles
provient uniquement des métadonnées des fichiers `*.jsonl` présents dans
`~/.codex/sessions` et `~/.claude/projects` au moment de la mesure ; aucun
contenu de session n'a été lu pour cet échantillon.
