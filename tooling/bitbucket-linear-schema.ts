import { z } from "zod";

const name = z.string().regex(/^[A-Za-z0-9][A-Za-z0-9_.-]*$/u);
const teamKey = z.string().regex(/^[A-Z][A-Z0-9]*$/u);
const repositoryConfig = z
  .strictObject({
    workspace: name,
    repository: name,
    teamKey,
    teamId: z.uuid(),
    completedStateId: z.uuid().optional(),
  })
  .readonly();
const configSchema = z
  .strictObject({
    bktContext: name,
    linearWorkspace: name,
    repositories: z.array(repositoryConfig).min(1).readonly(),
  })
  .readonly()
  .refine((config) => {
    const repositories = new Set<string>();
    const teams = new Map<string, string>();
    for (const repository of config.repositories) {
      const identity = `${repository.workspace}/${repository.repository}`;
      const teamSettings = JSON.stringify([
        repository.teamId,
        repository.completedStateId,
      ]);
      if (
        repositories.has(identity) ||
        (teams.has(repository.teamKey) &&
          teams.get(repository.teamKey) !== teamSettings)
      ) {
        return false;
      }
      repositories.add(identity);
      teams.set(repository.teamKey, teamSettings);
    }
    return (
      new Set(config.repositories.map((repository) => repository.teamId))
        .size === teams.size
    );
  }, "Duplicate repository or inconsistent team configuration");
const stateSchema = z
  .object({
    id: z.uuid(),
    name: z.string().min(1),
    type: z.enum([
      "triage",
      "backlog",
      "unstarted",
      "started",
      "completed",
      "canceled",
    ]),
  })
  .readonly();
const attachmentSchema = z
  .object({
    id: z.string().min(1),
    url: z.url(),
    title: z.string(),
    subtitle: z.string().nullable(),
  })
  .readonly();
type Config = z.infer<typeof configSchema>;
type RepositoryConfig = z.infer<typeof repositoryConfig>;
type State = z.infer<typeof stateSchema>;
type Attachment = z.infer<typeof attachmentSchema>;
type Team = Readonly<{
  id: string;
  key: string;
  states: readonly State[];
  completedStateId?: string;
}>;
type Issue = Readonly<{
  id: string;
  identifier: string;
  teamId: string;
  state: State;
  archivedAt: string | null;
  attachments: readonly Attachment[];
}>;
type PullRequest = Readonly<{
  id: number;
  title: string;
  description: string;
  branch: string;
  state: "OPEN" | "MERGED" | "DECLINED";
  url: string;
  repository: string;
  sourceRepository: string;
  sourceRepositoryId: string;
  destinationRepositoryId: string;
  authorId: string;
  teamKey: string;
  teamId: string;
}>;

export { attachmentSchema, configSchema, stateSchema };
export type {
  Attachment,
  Config,
  Issue,
  PullRequest,
  RepositoryConfig,
  State,
  Team,
};
