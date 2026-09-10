import { afterEach, expect, test } from "bun:test";
import {
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { installLumen } from "./install-lumen.ts";
import { join } from "node:path";
import { rejects } from "node:assert/strict";
import { tmpdir } from "node:os";

const directories: string[] = [];
const content = new TextEncoder().encode("abc");
const checksum =
  "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

async function fixture(): Promise<string> {
  const directory = await mkdtemp(join(tmpdir(), "lumen-test-"));
  directories.push(directory);
  return directory;
}

afterEach(async () => {
  await Promise.all(
    directories.splice(0).map(async (directory) => {
      await rm(directory, { recursive: true, force: true });
    }),
  );
});

async function verify(application: string): Promise<void> {
  if (
    (await readFile(join(application, "application"), "utf8")) !== "complete"
  ) {
    throw new Error("Unknown application");
  }
}

async function extract(_image: string, destination: string): Promise<void> {
  await mkdir(destination);
  await writeFile(join(destination, "application"), "complete");
}

test("publishes a complete application and preserves it on replay", async () => {
  const directory = await fixture();
  const options = {
    directory,
    checksum,
    verify,
    download: (): Promise<Uint8Array> => Promise.resolve(content),
    extract,
  };
  await installLumen(options);
  expect(await readFile(join(directory, "Lumen.app/application"), "utf8")).toBe(
    "complete",
  );
  await installLumen({
    ...options,
    download: () => Promise.reject(new Error("offline")),
  });
  expect(await readdir(directory)).toEqual(["Lumen.app"]);
});

test("rejects corrupted downloads before exposing an application", async () => {
  const directory = await fixture();
  await rejects(
    installLumen({
      directory,
      checksum,
      verify,
      download: () => Promise.resolve(new Uint8Array()),
      extract,
    }),
    /SHA-256/u,
  );
  expect(await readdir(directory)).toEqual([]);
});

test("download failures leave no partial application", async () => {
  const directory = await fixture();
  await rejects(
    installLumen({
      directory,
      checksum,
      verify,
      download: () => Promise.reject(new Error("offline")),
      extract,
    }),
    /offline/u,
  );
  expect(await readdir(directory)).toEqual([]);
});

test("copy failures remove staging without publishing a partial application", async () => {
  const directory = await fixture();
  await rejects(
    installLumen({
      directory,
      checksum,
      verify,
      download: (): Promise<Uint8Array> => Promise.resolve(content),
      extract: async (image, destination) => {
        await extract(image, destination);
        throw new Error("disk full");
      },
    }),
    /disk full/u,
  );
  expect(await readdir(directory)).toEqual([]);
});

test("refuses an existing file at the application destination", async () => {
  const directory = await fixture();
  await writeFile(join(directory, "Lumen.app"), "existing");
  await rejects(
    installLumen({
      directory,
      checksum,
      verify,
      download: (): Promise<Uint8Array> => Promise.resolve(content),
      extract,
    }),
    /exists/u,
  );
  expect(await readFile(join(directory, "Lumen.app"), "utf8")).toBe("existing");
});

test("refuses a symlink without modifying its target", async () => {
  const directory = await fixture();
  const target = await fixture();
  await writeFile(join(target, "application"), "existing");
  await symlink(target, join(directory, "Lumen.app"));
  await rejects(
    installLumen({
      directory,
      checksum,
      verify,
      download: () => Promise.resolve(content),
      extract,
    }),
    /exists/u,
  );
  expect(await readFile(join(target, "application"), "utf8")).toBe("existing");
});

test("refuses an unknown application without replacing it", async () => {
  const directory = await fixture();
  await mkdir(join(directory, "Lumen.app"));
  await writeFile(join(directory, "Lumen.app/application"), "other app");
  await rejects(
    installLumen({
      directory,
      checksum,
      verify,
      download: () => Promise.resolve(content),
      extract,
    }),
    /Unknown/u,
  );
  expect(await readFile(join(directory, "Lumen.app/application"), "utf8")).toBe(
    "other app",
  );
});

test("signature rejection leaves no installed application", async () => {
  const directory = await fixture();
  await rejects(
    installLumen({
      directory,
      checksum,
      verify: () => Promise.reject(new Error("invalid signature")),
      download: () => Promise.resolve(content),
      extract,
    }),
    /signature/u,
  );
  expect(await readdir(directory)).toEqual([]);
});

test("a destination appearing during extraction is preserved", async () => {
  const directory = await fixture();
  await rejects(
    installLumen({
      directory,
      checksum,
      verify,
      download: () => Promise.resolve(content),
      extract: async (image, destination) => {
        await extract(image, destination);
        await mkdir(join(directory, "Lumen.app"));
        await writeFile(
          join(directory, "Lumen.app/application"),
          "concurrent app",
        );
      },
    }),
  );
  expect(await readFile(join(directory, "Lumen.app/application"), "utf8")).toBe(
    "concurrent app",
  );
  expect(await readdir(directory)).toEqual(["Lumen.app"]);
});
