import { createInterface } from "node:readline/promises";
import { z } from "zod";

type PopupQuestions = Readonly<{
  ask: (question: string) => Promise<string | null>;
  close: () => void;
}>;

function openPopupQuestions(): PopupQuestions {
  if (!process.stdin.isTTY || !process.stdout.isTTY) {
    throw new Error("Issue picker requires a terminal popup");
  }
  const terminal = createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: true,
  });
  let closed = false;
  terminal.on("close", () => {
    closed = true;
  });
  terminal.on("SIGINT", () => {
    terminal.close();
  });
  return {
    ask: (question) => askQuestion(terminal, question, closed),
    close: () => {
      terminal.close();
    },
  };
}

async function askQuestion(
  terminal: Readonly<
    Pick<ReturnType<typeof createInterface>, "on" | "off" | "question">
  >,
  question: string,
  closed: boolean,
): Promise<string | null> {
  if (closed) {
    return null;
  }
  const cancellation = new AbortController();
  const onClose = (): void => {
    cancellation.abort();
  };
  const onKeyPress = (_text: unknown, key: unknown): void => {
    const parsed = z.object({ name: z.string().optional() }).safeParse(key);
    if (!parsed.success || parsed.data.name === "escape") {
      cancellation.abort();
    }
  };
  process.stdin.on("keypress", onKeyPress);
  terminal.on("close", onClose);
  try {
    return await terminal.question(`${question}\n> `, {
      signal: cancellation.signal,
    });
  } catch (error) {
    if (cancellation.signal.aborted) {
      return null;
    }
    throw error;
  } finally {
    process.stdin.off("keypress", onKeyPress);
    terminal.off("close", onClose);
  }
}

export { openPopupQuestions };
export type { PopupQuestions };
