# Retrait de llama.cpp

## Objectif

Retirer llama.cpp de la configuration reproductible et de la machine, ainsi que ses données propres,
sans affecter `whisper-cpp`, Ollama ni leurs dépendances ou modèles.

## État constaté

- Le target `ai` du `Makefile` dépend de `llama-cpp`.
- Le target `llama-cpp` installe la formule Homebrew `llama.cpp`.
- Homebrew a installé `llama.cpp 10360`.
- Aucun processus, service `launchd`, Login Item ou service Homebrew llama.cpp n'est actif.
- `/Users/sebastien/Library/Caches/llama.cpp` est le seul cache propre détecté et il est vide.
- Aucun modèle `.gguf` ou `.ggml` n'a été détecté dans les emplacements indexés ou standards.
- `ggml` doit rester installé car `whisper-cpp` en dépend.

## Changement retenu

1. Retirer `llama-cpp` des dépendances du target `ai`.
2. Supprimer le target `llama-cpp` du `Makefile`.
3. Désinstaller la formule Homebrew `llama.cpp`.
4. Supprimer `/Users/sebastien/Library/Caches/llama.cpp` après avoir confirmé qu'il s'agit toujours
   du cache propre attendu.

Les caches Hugging Face, Ollama et whisper, ainsi que la formule `ggml`, restent hors périmètre.

## Gestion des erreurs

La suppression du cache s'arrête si son chemin ou son type a changé depuis l'audit. La
désinstallation doit signaler explicitement une formule déjà absente, sans entraîner la suppression
automatique de dépendances partagées.

## Vérification

- Inspecter le diff du `Makefile` et vérifier que `llama-cpp` n'y apparaît plus.
- Vérifier syntaxiquement le `Makefile` avec un dry-run limité à un target sans effet global.
- Confirmer l'absence de la formule `llama.cpp`, de ses binaires liés et de son cache.
- Confirmer que `whisper-cpp` et `ggml` restent installés.
