import { createHash } from "node:crypto";
import { dirname, join } from "node:path";
import { DictionaryInstallationError } from "./install-hunspell-dictionary-error.ts";
import {
  assertDirectories,
  existingDictionaryMatches,
  prepareSpellingDirectory,
  publishDictionary,
} from "./install-hunspell-dictionary-files.ts";

type Installation = Readonly<{
  destination: string;
  expectedChecksum: string;
  home: string;
  url: string;
}>;

function parseInstallation(
  arguments_: readonly string[],
  environment: NodeJS.ProcessEnv,
): Installation {
  const url = arguments_[0];
  const expectedChecksum = arguments_[1];
  const destination = arguments_[2];
  if (url === undefined || url === "") {
    throw new DictionaryInstallationError("missing source URL");
  }
  if (expectedChecksum === undefined || expectedChecksum === "") {
    throw new DictionaryInstallationError("missing SHA-256 checksum");
  }
  if (destination === undefined || destination === "") {
    throw new DictionaryInstallationError("missing destination path");
  }
  if (!/^[0-9a-f]{64}$/.test(expectedChecksum)) {
    throw new DictionaryInstallationError(
      `Invalid SHA-256 checksum: ${expectedChecksum}`,
      64,
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

async function download(url: string): Promise<Uint8Array> {
  let response: Response;
  try {
    response = await fetch(url);
  } catch {
    throw new DictionaryInstallationError(`Dictionary download failed: ${url}`);
  }
  if (!response.ok) {
    throw new DictionaryInstallationError(
      `Dictionary download failed with HTTP ${response.status}: ${url}`,
    );
  }
  try {
    return new Uint8Array(await response.arrayBuffer());
  } catch {
    throw new DictionaryInstallationError(`Dictionary download failed: ${url}`);
  }
}

function verifyChecksum(
  content: Uint8Array,
  expectedChecksum: string,
  url: string,
): void {
  const actualChecksum = createHash("sha256").update(content).digest("hex");
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
  if (existingMatch === true) return;
  if (existingMatch === false) {
    throw new DictionaryInstallationError(
      `Refusing to replace existing dictionary: ${installation.destination}`,
    );
  }
  const content = await download(installation.url);
  verifyChecksum(content, installation.expectedChecksum, installation.url);
  await publishDictionary(
    installation.destination,
    content,
    installation.expectedChecksum,
    directories,
  );
}

export async function runDictionaryInstallation(
  arguments_: readonly string[],
  environment: NodeJS.ProcessEnv,
): Promise<number> {
  try {
    await install(parseInstallation(arguments_, environment));
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
