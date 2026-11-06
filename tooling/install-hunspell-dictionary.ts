import {
  assertDirectories,
  existingDictionaryMatches,
  prepareSpellingDirectory,
  publishDictionary,
} from "./install-hunspell-dictionary-files.ts";
import { dirname, join } from "node:path";
import { DictionaryInstallationError } from "./install-hunspell-dictionary-error.ts";
import { createHash } from "node:crypto";

type Installation = Readonly<{
  destination: string;
  expectedChecksum: string;
  home: string;
  url: string;
}>;

type DownloadLimits = Readonly<{
  maximumBytes: number;
  timeoutMilliseconds: number;
}>;

const sha256HexLength = 64;
const usageFailureExitCode = 64;
const maximumDownloadBytes = 33_554_432;
const downloadTimeoutMilliseconds = 120_000;
const downloadLimits: DownloadLimits = {
  maximumBytes: maximumDownloadBytes,
  timeoutMilliseconds: downloadTimeoutMilliseconds,
};

function parseInstallation(
  commandArguments: readonly string[],
  environment: Readonly<NodeJS.ProcessEnv>,
): Installation {
  const [url, expectedChecksum, destination] = commandArguments;
  if (url === undefined || url === "") {
    throw new DictionaryInstallationError("missing source URL");
  }
  if (expectedChecksum === undefined || expectedChecksum === "") {
    throw new DictionaryInstallationError("missing SHA-256 checksum");
  }
  if (destination === undefined || destination === "") {
    throw new DictionaryInstallationError("missing destination path");
  }
  if (
    expectedChecksum.length !== sha256HexLength ||
    !/^[0-9a-f]+$/u.test(expectedChecksum)
  ) {
    throw new DictionaryInstallationError(
      `Invalid SHA-256 checksum: ${expectedChecksum}`,
      usageFailureExitCode,
    );
  }
  const home = environment.HOME;
  if (home === undefined || home === "") {
    throw new DictionaryInstallationError("missing HOME");
  }
  const spellingDirectory = join(home, "Library", "Spelling");
  if (dirname(destination) !== spellingDirectory) {
    throw new DictionaryInstallationError(
      `Refusing dictionary destination outside ${spellingDirectory}: ${destination}`,
    );
  }
  return { destination, expectedChecksum, home, url };
}

async function readWithinLimit(
  body: Readonly<ReadableStream<Uint8Array>> | null,
  limits: DownloadLimits,
  url: string,
): Promise<Uint8Array> {
  const chunks: Uint8Array[] = [];
  let size = 0;
  for await (const chunk of body ?? []) {
    size += chunk.length;
    if (size > limits.maximumBytes) {
      throw new DictionaryInstallationError(
        `Dictionary download exceeds ${limits.maximumBytes} bytes: ${url}`,
      );
    }
    chunks.push(chunk);
  }
  return Uint8Array.from(Buffer.concat(chunks));
}

async function download(
  url: string,
  limits: DownloadLimits,
): Promise<Uint8Array> {
  const signal = AbortSignal.timeout(limits.timeoutMilliseconds);
  try {
    const response = await fetch(url, { signal });
    if (!response.ok) {
      throw new DictionaryInstallationError(
        `Dictionary download failed with HTTP ${response.status}: ${url}`,
      );
    }
    return await readWithinLimit(response.body, limits, url);
  } catch (error) {
    if (error instanceof DictionaryInstallationError) {
      throw error;
    }
    throw new DictionaryInstallationError(
      signal.aborted
        ? `Dictionary download timed out after ${limits.timeoutMilliseconds} ms: ${url}`
        : `Dictionary download failed: ${url}`,
    );
  }
}

function verifyChecksum(
  content: Readonly<ArrayLike<number>>,
  expectedChecksum: string,
  url: string,
): void {
  const actualChecksum = createHash("sha256")
    .update(Uint8Array.from(content))
    .digest("hex");
  if (actualChecksum !== expectedChecksum) {
    throw new DictionaryInstallationError(`SHA-256 mismatch for ${url}`);
  }
}

async function install(installation: Installation): Promise<void> {
  const directories = await prepareSpellingDirectory(installation.home);
  const existingMatch = await existingDictionaryMatches(
    installation.destination,
    installation.expectedChecksum,
  );
  await assertDirectories(directories);
  if (existingMatch === true) {
    return;
  }
  if (existingMatch === false) {
    throw new DictionaryInstallationError(
      `Refusing to replace existing dictionary: ${installation.destination}`,
    );
  }
  const content = await download(installation.url, downloadLimits);
  verifyChecksum(content, installation.expectedChecksum, installation.url);
  await publishDictionary(
    {
      content,
      destination: installation.destination,
      expectedChecksum: installation.expectedChecksum,
    },
    directories,
  );
}

async function runDictionaryInstallation(
  commandArguments: readonly string[],
  environment: Readonly<NodeJS.ProcessEnv>,
): Promise<number> {
  try {
    await install(parseInstallation(commandArguments, environment));
    return 0;
  } catch (error) {
    const failure =
      error instanceof DictionaryInstallationError
        ? error
        : new DictionaryInstallationError(
            `unexpected installation failure: ${error instanceof Error ? error.message : String(error)}`,
          );
    process.stderr.write(`install-hunspell-dictionary: ${failure.message}\n`);
    return failure.exitCode;
  }
}

export { download, runDictionaryInstallation };
