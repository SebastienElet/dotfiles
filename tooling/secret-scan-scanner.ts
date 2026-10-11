import { join } from "node:path";
import { rm } from "node:fs/promises";
import { z } from "zod";

const findingsSchema = z.array(
  z.object({ File: z.string(), Commit: z.string(), RuleID: z.string() }),
);
const scanTimeoutSeconds = 300;

function scannerProcess(
  root: string,
  directory: string,
  args: readonly string[],
): Bun.SyncSubprocess<"pipe", "pipe"> {
  const env = Object.fromEntries(
    Object.entries(process.env).filter(
      (entry: readonly [string, string | undefined]) =>
        !entry[0].startsWith("GITLEAKS_"),
    ),
  );
  try {
    return Bun.spawnSync(
      [
        "gitleaks",
        ...args,
        "--config",
        join(directory, "config.toml"),
        "--gitleaks-ignore-path",
        join(directory, "ignore"),
        "--ignore-gitleaks-allow",
        "--redact=100",
        "--no-banner",
        "--no-color",
        "--log-level=error",
        `--timeout=${scanTimeoutSeconds}`,
        "--report-format=json",
        "--report-path",
        join(directory, "report.json"),
      ],
      { cwd: root, env, stdin: "ignore", stdout: "pipe", stderr: "pipe" },
    );
  } catch {
    throw new Error(
      "Secret scan: Gitleaks is unavailable; run moon run repository:secret-prepare before committing.",
    );
  }
}

async function readFindings(
  report: string,
): Promise<z.infer<typeof findingsSchema>> {
  try {
    const json: unknown = await Bun.file(report).json();
    return findingsSchema.parse(json);
  } catch {
    throw new Error(
      "Secret scan: Gitleaks failed or returned an invalid report; check its installation and versioned configuration.",
    );
  }
}

async function runScanner(
  root: string,
  directory: string,
  args: readonly string[],
): Promise<void> {
  const report = join(directory, "report.json");
  await rm(report, { force: true });
  const result = scannerProcess(root, directory, args);
  const findings = await readFindings(report);
  for (const finding of findings) {
    process.stderr.write(
      `${JSON.stringify({ file: finding.File, commit: finding.Commit, type: finding.RuleID })}\n`,
    );
  }
  if (result.exitCode !== 0 || findings.length > 0) {
    throw new Error(
      `Secret scan refused (Gitleaks status ${result.exitCode}, ${findings.length} findings).`,
    );
  }
}

export { runScanner };
