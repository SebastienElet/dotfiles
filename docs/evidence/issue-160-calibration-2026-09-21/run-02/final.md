Correction appliquée dans [src/trim-label.js](/synthetic/issue160-calibration/run-02/workspace/src/trim-label.js:1) : la chaîne vide reste désormais `""`.

Contrôles locaux macOS, Node.js 26.8.1 :

- `npm test` : 3/3 réussis
- `git diff --check` : réussi
- Aucun commentaire ajouté

Changement livrable, sous réserve de la CI requise sur les cibles annoncées Node.js 24/Linux.
