# Exceptions aux sources Homebrew

Ce document est l'inventaire opérationnel des installations que Homebrew Bundle ne porte pas,
référencé par [ADR-002](adr/002-homebrew-source-unique.md).

| Source                        | Version                            | Intégrité                                    | Mise à jour                            | Rejeu                                               |
| ----------------------------- | ---------------------------------- | -------------------------------------------- | -------------------------------------- | --------------------------------------------------- |
| Amorçage Homebrew             | Installateur officiel `HEAD`       | HTTPS Homebrew                               | Gérée par Homebrew après installation  | Seulement si `brew` est absent                      |
| Paquets Node via Volta ou npm | Coordonnée de la cible             | Vérification du registre par le gestionnaire | `tooling/upgrade`                      | Sentinelle du shim ou du binaire                    |
| Plugins Fisher et TPM         | Révision distante à l'installation | Objets GitHub en HTTPS                       | Commande explicite du gestionnaire     | Configuration Fisher ou checkout TPM                |
| Claude Code                   | Canal éditeur `latest`             | HTTPS éditeur                                | `claude update`                        | Sentinelle du binaire                               |
| Moon                          | Canal éditeur `latest`             | HTTPS éditeur                                | `tooling/upgrade` → `moon upgrade`     | Sentinelle du binaire                               |
| Images Docker des MCP         | Image et tag déclarés              | Manifeste et couches du registre             | `docker pull` ou `docker compose pull` | Image locale réutilisée                             |
| Thèmes Bat                    | Branche Catppuccin par défaut      | HTTPS GitHub                                 | Changement explicite de source         | Fichier de thème existant                           |
| Dictionnaires Hunspell        | Commit LibreOffice déclaré         | SHA-256 déclaré                              | Changement du commit et du checksum    | Destination identique conservée, divergence refusée |

Les sources internes à un outil local appelé par Moon ou le `Makefile` relèvent des tests de cet outil.
