# Synchronisation des configurations natives

`arnes sync config --agent <agent> --scope user` applique uniquement les valeurs
`user_config` déclarées dans le manifeste. Le scope projet retourne `unsupported`.
Les autres valeurs du fichier, notamment hooks, serveurs MCP et statusline, sont
conservées. Une valeur sélectionnée dont le type natif est incompatible est refusée.

| Agent  | Configuration utilisateur | MCP utilisateur      | MCP projet           |
| ------ | ------------------------- | -------------------- | -------------------- |
| Claude | `.claude/settings.json`   | `.claude.json`       | `.mcp.json`          |
| Cursor | `.cursor/cli-config.json` | `.cursor/mcp.json`   | `.cursor/mcp.json`   |
| Codex  | `.codex/config.toml`      | `.codex/config.toml` | `.codex/config.toml` |

Les chemins utilisateur sont relatifs à HOME ; les chemins projet sont relatifs au
répertoire courant retenu par Arnes. Les sources canoniques du dépôt courant et du
dépôt de déploiement sont protégées. Les parents symboliques, fichiers symboliques
ou liens physiques multiples sont refusés. Les fichiers JSON ambigus ou malformés
sont refusés ; les valeurs non sélectionnées conservent leurs nombres exacts grâce
à [serde_json RawValue](https://docs.rs/serde_json/latest/serde_json/value/struct.RawValue.html).
Une modification peut changer la présentation du fichier. Une configuration déjà
conforme conserve ses octets et son inode.

`arnes sync mcp --agent <agent> --scope <scope>` synchronise les déclarations MCP
locales du manifeste : commande, arguments ordonnés, références d'environnement et
état `enabled` lorsque le lecteur natif le confirme. Aucun serveur n'est exécuté.
Une commande indisponible, une collision avec l'autre scope ou une entrée dont les
champs sélectionnés ne sont pas interprétables sont refusées avant publication.
Les entrées étrangères et les sections non sélectionnées sont conservées.

Les entrées JSON publiées utilisent le transport `stdio`. Claude référence
l'environnement avec `${NAME}` ; Cursor utilise `${env:NAME}` ; Codex utilise
`env_vars`. Les valeurs littérales et expressions avec valeur par défaut ne sont
pas traitées comme des références conformes. Elles ne sont pas reproduites dans
les diagnostics. Claude projet représente `enabled` dans
`HOME/.claude.json`, sous `projects[chemin_du_projet].disabledMcpServers`.
Claude utilisateur avec `enabled: false` retourne `unsupported` (code 1).
Pour Cursor, toute déclaration `enabled` est rejetée lors de la validation du
manifeste (code 2) : son lecteur natif ne représente pas cet état. Ces cas ne
provoquent aucune mutation.

Une entrée absente peut être créée. Une entrée conforme reste `current`, sans
adoption. Pour modifier une entrée divergente, Arnes exige un reçu correspondant à
son contenu et à son état natif, sous
`.arnes-mcp-ownership/<agent>-<scope>-<nom>.json` dans la racine du scope sélectionné.
Le reçu contient une empreinte de l'entrée, sa racine et l'identité agent/scope/nom ;
il ne conserve pas l'inode ni les valeurs d'environnement. Un changement local invalide cette
preuve de propriété. Copier un reçu entre racines ne transfère pas la propriété.

Chaque fichier est publié atomiquement après vérification de son état observé.
L'ordre MCP est : configuration, préférences Claude projet lorsqu'elles changent,
puis reçu. Ces fichiers ne forment pas une transaction globale. Un échec de
préférences ou de reçu retourne `failed` et peut laisser la configuration publiée.
Un rejeu conforme reste `current` sans créer de reçu manquant ; une divergence
ultérieure sans reçu correspondant est refusée. Aucune suppression n'est effectuée.
Une sélection vide ou un refus retourne un code non nul ; `failed` retourne 2.

Les formats sont vérifiés dans les documentations officielles de
[Claude settings](https://code.claude.com/docs/en/settings),
[Claude MCP](https://code.claude.com/docs/en/mcp),
[Cursor CLI configuration](https://cursor.com/docs/cli/reference/configuration),
[Cursor MCP](https://cursor.com/docs/mcp),
[Cursor CLI MCP](https://cursor.com/docs/cli/mcp) et
[Codex configuration](https://developers.openai.com/codex/config-reference/).
Les tests CLI d'Arnes et son Doctor vérifient les projections natives dans des
fixtures ; ils ne lancent pas ces trois agents.
