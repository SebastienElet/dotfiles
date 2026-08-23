import { dlopen } from "bun:ffi";

const renameExclusive = 4;
const renameNoReplace = 1;
const currentWorkingDirectory = -100;

export function publishDirectoryExclusively(
  source: string,
  destination: string,
): void {
  if (process.platform === "darwin") {
    const library = dlopen("/usr/lib/libSystem.B.dylib", {
      renamex_np: { args: ["cstring", "cstring", "u32"], returns: "i32" },
    });
    try {
      if (
        library.symbols.renamex_np(source, destination, renameExclusive) === 0
      )
        return;
    } finally {
      library.close();
    }
  } else if (process.platform === "linux") {
    const library = dlopen("libc.so.6", {
      renameat2: {
        args: ["i32", "cstring", "i32", "cstring", "u32"],
        returns: "i32",
      },
    });
    try {
      if (
        library.symbols.renameat2(
          currentWorkingDirectory,
          source,
          currentWorkingDirectory,
          destination,
          renameNoReplace,
        ) === 0
      )
        return;
    } finally {
      library.close();
    }
  } else {
    throw new Error(`unsupported platform for exclusive publication`);
  }
  throw new Error(
    `attachment destination already exists or cannot be published`,
  );
}
