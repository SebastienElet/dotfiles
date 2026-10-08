import { z } from "zod";

const dockerImages = {
  scrapling:
    "pyd4vinci/scrapling:0.4.14@sha256:48f5938fff7c0684e11995f5cdc6497ce13d9ab0d914e58e9ff5f2055099d284",
  cloakbrowser:
    "cloakhq/cloakbrowser:0.5.3@sha256:6bb92c481c700bc68243becd219408ef86e300807906038d6873780bdc9ffaef",
} as const;
const imageIdentifierSchema = z.string().regex(/^sha256:[0-9a-f]{64}$/u);
const repositoryComponent = "[a-z0-9]+(?:(?:[._]|__|-+)[a-z0-9]+)*";
const registry = "(?:[a-z0-9]+(?:[.-][a-z0-9]+)*(?::[0-9]+)?/)?";
const imageReferenceSchema = z
  .string()
  .regex(
    new RegExp(
      `^${registry}${repositoryComponent}(?:/${repositoryComponent})*(?::[A-Za-z0-9_][A-Za-z0-9_.-]{0,127})?@sha256:[0-9a-f]{64}$`,
      "u",
    ),
    "invalid immutable Docker image reference (expected name[:tag]@sha256:<64 lowercase hex digits>)",
  );
const imageInspectionSchema = z.object({
  Id: imageIdentifierSchema,
  RepoDigests: z.array(imageReferenceSchema),
});

function canonicalRepository(repository: string): string {
  const parts = repository.split("/");
  const first = parts[0] ?? "";
  const explicitRegistry =
    parts.length > 1 &&
    (first.includes(".") || first.includes(":") || first === "localhost");
  const host = explicitRegistry ? first : "docker.io";
  const path = explicitRegistry ? parts.slice(1).join("/") : repository;
  const normalizedHost = host === "index.docker.io" ? "docker.io" : host;
  return `${normalizedHost}/${normalizedHost === "docker.io" && !path.includes("/") ? "library/" : ""}${path}`;
}

function repositoryDigest(reference: string): string {
  const [taggedRepository = "", digest = ""] = reference.split("@");
  const repository = taggedRepository.replace(/:[^/:]+$/u, "");
  return `${canonicalRepository(repository)}@${digest}`;
}

function parseImageIdentity(output: string, reference: string): string {
  const inspection = imageInspectionSchema.parse(JSON.parse(output));
  const expected = repositoryDigest(reference);
  if (
    !inspection.RepoDigests.some(
      (digest) => repositoryDigest(digest) === expected,
    )
  ) {
    throw new Error(`Docker image identity differs from expected ${reference}`);
  }
  return inspection.Id;
}

export {
  dockerImages,
  imageReferenceSchema,
  parseImageIdentity,
  repositoryDigest,
};
