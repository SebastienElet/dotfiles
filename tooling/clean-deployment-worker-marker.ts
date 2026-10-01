import { lstatSync, renameSync, unlinkSync, writeFileSync } from "node:fs";

function publishWorkerMarker(
  path: string,
  state: Readonly<Record<string, unknown>>,
): void {
  const temporary = `${path}.${process.pid}.tmp`;
  let created = false;
  try {
    writeFileSync(temporary, JSON.stringify(state), { flag: "wx" });
    created = true;
    renameSync(temporary, path);
  } finally {
    if (
      created &&
      lstatSync(temporary, { throwIfNoEntry: false })?.isFile() === true
    ) {
      unlinkSync(temporary);
    }
  }
}

export { publishWorkerMarker };
