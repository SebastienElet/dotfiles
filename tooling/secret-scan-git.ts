import { z } from "zod";

const nulPaths = z.array(z.string().min(1));

function gitOutput(root: string, args: readonly string[]): Buffer {
  const result = Bun.spawnSync(["git", "-C", root, ...args], {
    stdin: "ignore",
    stdout: "pipe",
    stderr: "pipe",
  });
  if (result.exitCode !== 0) {
    throw new Error(
      `Secret scan: Git ${args[0]} failed; check the repository and required refs.`,
    );
  }
  return result.stdout;
}

function gitPaths(root: string, args: readonly string[]): string[] {
  const output = gitOutput(root, args).toString();
  if (output === "") {
    return [];
  }
  if (!output.endsWith("\0")) {
    throw new Error("Secret scan: incomplete Git path list.");
  }
  return nulPaths.parse(output.slice(0, -1).split("\0"));
}

function rejectPrivatePaths(paths: readonly string[]): void {
  const templates = new Set([".env.example", ".env.sample", ".env.template"]);
  const forbidden = paths.filter((path) => {
    const name = (path.split("/").at(-1) ?? "").toLowerCase();
    const privatePath = path.toLowerCase();
    return (
      ((name === ".env" || name.startsWith(".env.")) && !templates.has(path)) ||
      ["id_rsa", "id_ed25519", "id_ecdsa", "id_dsa"].includes(name) ||
      name.endsWith(".p12") ||
      name.endsWith(".pfx") ||
      privatePath === "home/.aws/credentials" ||
      privatePath === "home/.config/gcloud/application_default_credentials.json"
    );
  });
  if (forbidden.length > 0) {
    throw new Error(
      `Private paths refused: ${forbidden.map((path) => JSON.stringify(path)).join(", ")}`,
    );
  }
}

function commitSha(root: string, ref: string): string {
  return z
    .string()
    .regex(/^[a-f0-9]{40}$/u)
    .parse(
      gitOutput(root, ["rev-parse", "--verify", `${ref}^{commit}`])
        .toString()
        .trim(),
    );
}

export { commitSha, gitOutput, gitPaths, rejectPrivatePaths };
