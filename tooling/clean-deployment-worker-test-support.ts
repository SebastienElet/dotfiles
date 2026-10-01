import {
  chmodSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import {
  createDeploymentFixture,
  runDeploymentHelper,
} from "./deployment-test-support.ts";
import { dirname, join } from "node:path";
import { z } from "zod";

function fixture(): ReturnType<typeof createDeploymentFixture> {
  const context = createDeploymentFixture("clean-retired-remem");
  mkdirSync(join(context.repository, "home"));
  writeFileSync(
    join(context.repository, "home/.arnes.yaml"),
    "version: 1\nskills: []\n",
  );
  return context;
}

type Fixture = ReturnType<typeof fixture>;
const executableMode = 0o755;

function clean(context: Fixture): ReturnType<typeof runDeploymentHelper> {
  return runDeploymentHelper(context, {
    helper: "clean-deployment.ts",
    arguments: [context.repository, context.home, "--apply"],
  });
}

function managedLink(
  context: Fixture,
  path: string,
  sourcePath: string,
): Readonly<{ destination: string; source: string }> {
  const destination = join(context.home, path);
  const source = join(context.repository, sourcePath);
  mkdirSync(dirname(destination), { recursive: true });
  symlinkSync(source, destination);
  return { destination, source };
}

function fixtureWorkerDefinition(home: string): Readonly<{
  Label: string;
  ProgramArguments: readonly string[];
  EnvironmentVariables: Readonly<{ PATH: string }>;
  RunAtLoad: boolean;
  StartInterval: number;
  ProcessType: string;
}> {
  return {
    Label: "dev.remem.worker",
    ProgramArguments: [join(home, ".local/bin/remem"), "worker", "--once"],
    EnvironmentVariables: {
      PATH: `${home}/.local/bin:${home}/.volta/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin`,
    },
    RunAtLoad: true,
    StartInterval: 300,
    ProcessType: "Background",
  };
}

function installWorkerProviders(context: Fixture): void {
  for (const command of ["osascript", "plutil", "launchctl"]) {
    const executable = join(context.bin, command);
    writeFileSync(
      executable,
      `#!/usr/bin/env bun\nimport ${JSON.stringify(join(import.meta.dir, "clean-deployment-worker-test-provider.ts"))};\n`,
    );
    chmodSync(executable, executableMode);
  }
}

function workerFixture(
  options: Readonly<{
    loaded?: boolean;
    foreignDisk?: boolean;
    foreignService?: boolean;
    conflictingProgram?: boolean;
    failure?: string;
  }> = {},
): Fixture {
  const context = fixture();
  const home = realpathSync(context.home);
  const definition = fixtureWorkerDefinition(home);
  const config = managedLink(
    context,
    ".remem/config.toml",
    "home/.remem/config.toml",
  );
  mkdirSync(dirname(config.source), { recursive: true });
  writeFileSync(config.source, "owned config\n");
  const plist = join(home, "Library/LaunchAgents/dev.remem.worker.plist");
  mkdirSync(dirname(plist), { recursive: true });
  writeFileSync(
    plist,
    JSON.stringify(
      options.foreignDisk === true
        ? { ...definition, ProgramArguments: ["foreign", "worker", "--once"] }
        : definition,
    ),
  );
  const state = {
    definition:
      options.loaded === false
        ? null
        : {
            Label: definition.Label,
            Program:
              options.conflictingProgram === true ? "foreign" : undefined,
            ProgramArguments:
              options.foreignService === true
                ? ["foreign", "worker", "--once"]
                : definition.ProgramArguments,
          },
    failure: options.failure,
    trace: [],
  };
  writeFileSync(join(context.root, "worker-state.json"), JSON.stringify(state));
  installWorkerProviders(context);
  return context;
}

function workerClean(
  context: Fixture,
  apply = true,
): ReturnType<typeof runDeploymentHelper> {
  return runDeploymentHelper(
    context,
    {
      helper: "clean-deployment.ts",
      arguments: [
        context.repository,
        context.home,
        ...(apply ? ["--apply"] : []),
      ],
    },
    {
      PATH: `${context.bin}:${dirname(process.execPath)}:/usr/bin:/bin`,
      CLEAN_WORKER_TEST_STATE: join(context.root, "worker-state.json"),
    },
  );
}

function workerState(context: Fixture): Readonly<{
  definition: unknown;
  trace: readonly Readonly<{ command: string; configPresent: boolean }>[];
}> {
  return z
    .object({
      definition: z.unknown(),
      trace: z
        .array(
          z
            .object({ command: z.string(), configPresent: z.boolean() })
            .readonly(),
        )
        .readonly(),
    })
    .readonly()
    .parse(
      JSON.parse(readFileSync(join(context.root, "worker-state.json"), "utf8")),
    );
}

function workerPlist(context: Fixture): string {
  return join(context.home, "Library/LaunchAgents/dev.remem.worker.plist");
}

export {
  clean,
  fixture,
  managedLink,
  workerClean,
  workerFixture,
  workerPlist,
  workerState,
};
