import { dirname, isAbsolute, parse, relative, resolve, sep } from "node:path";
import { lstatSync, realpathSync } from "node:fs";

function within(root: string, path: string): boolean {
  const difference = relative(root, path);
  return (
    difference === "" ||
    (!difference.startsWith(`..${sep}`) &&
      difference !== ".." &&
      !isAbsolute(difference))
  );
}

function cleanRoots(
  repository: string,
  home: string,
): Readonly<{ repository: string; home: string }> {
  if (!isAbsolute(repository) || !isAbsolute(home)) {
    throw new Error("Repository and HOME must be absolute paths");
  }
  const roots = {
    repository: realpathSync(repository),
    home: realpathSync(home),
  };
  if (
    roots.home === parse(roots.home).root ||
    within(roots.repository, roots.home)
  ) {
    throw new Error(`Unsafe cleanup HOME: ${home}`);
  }
  if (
    !lstatSync(roots.home).isDirectory() ||
    !lstatSync(roots.repository).isDirectory()
  ) {
    throw new Error("Repository and HOME must be directories");
  }
  return roots;
}

function canonicalLinkTarget(path: string): string {
  let parent = dirname(path);
  while (lstatSync(parent, { throwIfNoEntry: false }) === undefined) {
    const next = dirname(parent);
    if (next === parent) {
      throw new Error(`No existing parent for link target: ${path}`);
    }
    parent = next;
  }
  return resolve(realpathSync(parent), relative(parent, path));
}

function verifyParents(home: string, repository: string, path: string): void {
  if (path === home || !within(home, path) || within(repository, path)) {
    throw new Error(`Unsafe cleanup destination: ${path}`);
  }
  let parent = dirname(path);
  while (parent !== home) {
    if (!within(home, parent) || parent === dirname(parent)) {
      throw new Error(`Cleanup parent walk escaped HOME: ${path}`);
    }
    const metadata = lstatSync(parent, { throwIfNoEntry: false });
    if (metadata?.isSymbolicLink() === true) {
      throw new Error(`Unsafe symbolic parent: ${parent}`);
    }
    if (metadata !== undefined && !metadata.isDirectory()) {
      throw new Error(`Cleanup parent is not a directory: ${parent}`);
    }
    parent = resolve(parent, "..");
  }
}

export { canonicalLinkTarget, cleanRoots, verifyParents };
