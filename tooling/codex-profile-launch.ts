import { codexGitArguments } from "./codex-profile-git.ts";
import { profileSchema } from "./codex-profile-observation.ts";
import { startProfileRelay } from "./codex-profile-relay.ts";
import { z } from "zod";

const hookTimeoutSeconds = 5;
const terminationTimeoutMilliseconds = 1000;
const events = ["SessionStart", "PreToolUse", "Stop"] as const;

function shellQuote(value: string): string {
  return `'${value.replaceAll("'", String.raw`'\''`)}'`;
}

async function supervise(
  arguments_: readonly string[],
  relayPath: string,
): Promise<number> {
  const child = Bun.spawn([...arguments_], {
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit",
    env: {
      ...process.env,
      DOTFILES_CODEX_PROFILE_RELAY: relayPath,
      TERM: "dumb",
    },
  });
  let termination: ReturnType<typeof setTimeout> | null = null;
  const terminate = (): void => {
    if (termination === null) {
      child.kill("SIGTERM");
      termination = setTimeout(() => {
        child.kill("SIGKILL");
      }, terminationTimeoutMilliseconds);
      termination.unref();
    }
  };
  const interrupt = (): void => {
    child.kill("SIGINT");
  };
  process.on("SIGINT", interrupt);
  process.on("SIGTERM", terminate);
  process.on("SIGHUP", terminate);
  try {
    return await child.exited;
  } finally {
    if (termination !== null) {
      clearTimeout(termination);
    }
    process.off("SIGINT", interrupt);
    process.off("SIGTERM", terminate);
    process.off("SIGHUP", terminate);
  }
}

async function launchProfile(
  arguments_: readonly string[],
  entry: string,
): Promise<number> {
  const [model, effort, ...forwarded] = z
    .tuple([profileSchema.shape.model, profileSchema.shape.effort])
    .rest(z.string())
    .parse(arguments_);
  const executable = Bun.which("codex");
  if (executable === null) {
    throw new Error("Codex unavailable");
  }
  const relay = await startProfileRelay();
  try {
    const command = [
      process.execPath,
      "--config=/dev/null",
      "--no-env-file",
      entry,
      "hook",
      model,
      effort,
    ]
      .map((value) => shellQuote(value))
      .join(" ");
    const hooks = events.flatMap((event) => [
      "-c",
      `hooks.${event}=[{hooks=[{type="command",command=${JSON.stringify(command)},timeout=${hookTimeoutSeconds}}]}]`,
    ]);
    process.stderr.write(
      `codex-profile: inconnu; demandé ${model}/${effort}; observation en attente des hooks approuvés\n`,
    );
    return await supervise(
      [
        executable,
        "-c",
        `model=${JSON.stringify(model)}`,
        "-c",
        `model_reasoning_effort=${JSON.stringify(effort)}`,
        ...hooks,
        ...codexGitArguments(forwarded),
        ...forwarded,
      ],
      relay.path,
    );
  } finally {
    await relay.close();
  }
}

export { launchProfile };
