import { resolve } from "node:path";

const source = resolve(process.cwd(), "source.ts");
const directive = ["// oxlint", "-disable-next-line"].join("");
await Bun.write(
  source,
  `${directive} typescript/no-floating-promises\nPromise.resolve();\n`,
);
