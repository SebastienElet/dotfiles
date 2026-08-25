import { dlopen } from "bun:ffi";

const currentWorkingDirectory = -100;
const renameExclusive = 4;
const renameNoReplace = 1;
const successfulExitCode = 0;
const publishOnDarwin = (source: string, destination: string): boolean => {
  const library = dlopen("/usr/lib/libSystem.B.dylib", {
    renamex_np: { args: ["cstring", "cstring", "u32"], returns: "i32" },
  });
  try {
    return (
      library.symbols.renamex_np(source, destination, renameExclusive) ===
      successfulExitCode
    );
  } finally {
    library.close();
  }
};
const publishOnLinux = (source: string, destination: string): boolean => {
  const library = dlopen("libc.so.6", {
    renameat2: {
      args: ["i32", "cstring", "i32", "cstring", "u32"],
      returns: "i32",
    },
  });
  try {
    return (
      library.symbols.renameat2(
        currentWorkingDirectory,
        source,
        currentWorkingDirectory,
        destination,
        renameNoReplace,
      ) === successfulExitCode
    );
  } finally {
    library.close();
  }
};

export const publishDirectoryExclusively = (
  source: string,
  destination: string,
): void => {
  if (process.platform === "darwin" && publishOnDarwin(source, destination)) {
    return;
  }
  if (process.platform === "linux" && publishOnLinux(source, destination)) {
    return;
  }
  if (process.platform !== "darwin" && process.platform !== "linux") {
    throw new Error(`unsupported platform for exclusive publication`);
  }
  throw new Error(
    `attachment destination already exists or cannot be published`,
  );
};
