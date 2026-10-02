import { afterEach, expect, test } from "bun:test";
import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { installAppleNotesExporter } from "./install-apple-notes-exporter.ts";
import { join } from "node:path";
import { rejects } from "node:assert/strict";
import { tmpdir } from "node:os";

const executableMode = 0o755;
const regularMode = 0o644;
const directories: string[] = [];
const applicationName = "Apple Notes Exporter.app";
const bundleInfo = {
  CFBundleIdentifier: "com.zaremski.AppleNotesExporter",
  CFBundleShortVersionString: "2.0",
  CFBundleVersion: "2",
  CFBundleExecutable: "Apple Notes Exporter",
};

async function fixture(): Promise<string> {
  const directory = await mkdtemp(join(tmpdir(), "apple-notes-exporter-test-"));
  directories.push(directory);
  return directory;
}

afterEach(async () => {
  await Promise.all(
    directories
      .splice(0)
      .map((directory) => rm(directory, { recursive: true, force: true })),
  );
});

async function bundle(directory: string): Promise<string> {
  const application = join(directory, applicationName);
  await mkdir(join(application, "Contents/MacOS"), { recursive: true });
  await mkdir(join(application, "Contents/SharedSupport"));
  await writeFile(
    join(application, "Contents/Info.plist"),
    JSON.stringify(bundleInfo),
  );
  for (const executable of [
    "MacOS/Apple Notes Exporter",
    "SharedSupport/notes-export-mcp",
  ]) {
    const path = join(application, "Contents", executable);
    await writeFile(path, "application");
    await chmod(path, executableMode);
  }
  return application;
}

function options(
  directory: string,
): Parameters<typeof installAppleNotesExporter>[0] {
  return {
    directory,
    download: (archive: string): Promise<void> => writeFile(archive, "archive"),
    extract: async (_archive: string, staging: string): Promise<void> => {
      await bundle(staging);
    },
    readBundleInfo: async (path: string): Promise<unknown> =>
      JSON.parse(await readFile(path, "utf8")),
  };
}

test("installs the complete bundle, ignores unrelated extracted files, and preserves replay", async () => {
  const directory = await fixture();
  const installation = options(directory);
  await installAppleNotesExporter({
    ...installation,
    extract: async (archive, staging) => {
      await installation.extract(archive, staging);
      await writeFile(join(staging, "unrelated"), "unrelated");
    },
  });
  const application = join(directory, applicationName);
  expect(
    await readFile(
      join(application, "Contents/SharedSupport/notes-export-mcp"),
      "utf8",
    ),
  ).toBe("application");
  const replayDownload = (): Promise<void> =>
    Promise.reject(new Error("offline"));
  await installAppleNotesExporter({
    ...installation,
    download: replayDownload,
  });
  expect(await readdir(directory)).toEqual([applicationName]);
});

for (const phase of ["download", "extract"] as const) {
  test(`${phase} failure leaves no application or staging`, async () => {
    const directory = await fixture();
    const installation = options(directory);
    await rejects(
      installAppleNotesExporter({
        ...installation,
        [phase]: () => Promise.reject(new Error(`${phase} failed`)),
      }),
      new RegExp(`${phase} failed`, "u"),
    );
    expect(await readdir(directory)).toEqual([]);
  });
}

test("refuses a regular file at the destination", async () => {
  const directory = await fixture();
  await writeFile(join(directory, applicationName), "existing");
  await rejects(installAppleNotesExporter(options(directory)), /exists/u);
  expect(await readFile(join(directory, applicationName), "utf8")).toBe(
    "existing",
  );
});

test("refuses a symlink without changing its target", async () => {
  const directory = await fixture();
  const target = await fixture();
  await bundle(target);
  await symlink(
    join(target, applicationName),
    join(directory, applicationName),
  );
  await rejects(installAppleNotesExporter(options(directory)), /exists/u);
  expect(
    await readFile(
      join(target, applicationName, "Contents/Info.plist"),
      "utf8",
    ),
  ).toBe(JSON.stringify(bundleInfo));
});

for (const field of ["CFBundleIdentifier", "CFBundleExecutable"] as const) {
  test(`refuses divergent ${field} without modifying the installed bundle`, async () => {
    const directory = await fixture();
    const application = await bundle(directory);
    const info = JSON.stringify({ ...bundleInfo, [field]: "other" });
    await writeFile(join(application, "Contents/Info.plist"), info);
    await rejects(
      installAppleNotesExporter(options(directory)),
      /Invalid input/u,
    );
    expect(
      await readFile(join(application, "Contents/Info.plist"), "utf8"),
    ).toBe(info);
  });
}

for (const defect of ["missing", "not executable", "symlink"] as const) {
  test(`refuses a ${defect} MCP binary without publishing the staged application`, async () => {
    const directory = await fixture();
    const installation = options(directory);
    await rejects(
      installAppleNotesExporter({
        ...installation,
        extract: async (archive, staging) => {
          await installation.extract(archive, staging);
          const mcp = join(
            staging,
            applicationName,
            "Contents/SharedSupport/notes-export-mcp",
          );
          if (defect === "not executable") {
            await chmod(mcp, regularMode);
            return;
          }
          await rm(mcp);
          if (defect === "symlink") {
            await symlink(
              join(
                staging,
                applicationName,
                "Contents/MacOS/Apple Notes Exporter",
              ),
              mcp,
            );
          }
        },
      }),
      /MCP|Missing/u,
    );
    expect(await readdir(directory)).toEqual([]);
  });
}

test("propagates bundle metadata reader failures and cleans staging", async () => {
  const directory = await fixture();
  await rejects(
    installAppleNotesExporter({
      ...options(directory),
      readBundleInfo: () => Promise.reject(new Error("plutil failed")),
    }),
    /plutil failed/u,
  );
  expect(await readdir(directory)).toEqual([]);
});

test("refuses an incomplete extracted bundle", async () => {
  const directory = await fixture();
  await rejects(
    installAppleNotesExporter({
      ...options(directory),
      extract: () => Promise.resolve(),
    }),
    /Missing/u,
  );
  expect(await readdir(directory)).toEqual([]);
});

test("preserves a destination appearing during extraction", async () => {
  const directory = await fixture();
  const installation = options(directory);
  await rejects(
    installAppleNotesExporter({
      ...installation,
      extract: async (archive, staging) => {
        await installation.extract(archive, staging);
        await mkdir(join(directory, applicationName));
        await writeFile(
          join(directory, applicationName, "existing"),
          "concurrent",
        );
      },
    }),
    /appeared/u,
  );
  expect(
    await readFile(join(directory, applicationName, "existing"), "utf8"),
  ).toBe("concurrent");
  expect(await readdir(directory)).toEqual([applicationName]);
});

test("preserves an already installed valid newer version without downloading", async () => {
  const directory = await fixture();
  const application = await bundle(directory);
  const info = JSON.stringify({
    ...bundleInfo,
    CFBundleShortVersionString: "2.1",
    CFBundleVersion: "3",
  });
  await writeFile(join(application, "Contents/Info.plist"), info);
  await installAppleNotesExporter({
    ...options(directory),
    download: () => Promise.reject(new Error("offline")),
  });
  expect(await readFile(join(application, "Contents/Info.plist"), "utf8")).toBe(
    info,
  );
});

for (const field of [
  "CFBundleShortVersionString",
  "CFBundleVersion",
] as const) {
  test(`refuses unpinned ${field} in the downloaded bundle`, async () => {
    const directory = await fixture();
    const installation = options(directory);
    await rejects(
      installAppleNotesExporter({
        ...installation,
        extract: async (archive, staging) => {
          await installation.extract(archive, staging);
          await writeFile(
            join(staging, applicationName, "Contents/Info.plist"),
            JSON.stringify({ ...bundleInfo, [field]: "other" }),
          );
        },
      }),
      /Invalid input/u,
    );
    expect(await readdir(directory)).toEqual([]);
  });
}
