import { expect, test } from "bun:test";

const popupProbe =
  'import { openPopupQuestions } from "./popup-questions.ts"; const questions = openPopupQuestions(); const answer = await questions.ask("Choose provider> "); questions.close(); process.stdout.write(JSON.stringify({answer}));';

async function answerPopup(
  input: string,
): Promise<Readonly<{ status: number; output: string }>> {
  let output = "";
  let answered = false;
  const terminal = new Bun.Terminal({
    data: (
      pty: Readonly<Pick<Bun.Terminal, "write">>,
      bytes: Readonly<ArrayLike<number>>,
    ): void => {
      output += new TextDecoder().decode(Uint8Array.from(bytes));
      if (!answered && output.includes("Choose provider>")) {
        answered = true;
        pty.write(input);
      }
    },
  });
  const child = Bun.spawn(
    [
      process.execPath,
      "--config=/dev/null",
      "--no-env-file",
      "--eval",
      popupProbe,
    ],
    { cwd: import.meta.dir, terminal, timeout: 2000 },
  );
  const status = await child.exited;
  terminal.close();
  return { output, status };
}

test("reads a provider choice through a real terminal", async () => {
  const result = await answerPopup("2\r");

  expect(result.status).toBe(0);
  expect(result.output).toContain('"answer":"2"');
});

test("Escape cancels the real terminal prompt without choosing a default", async () => {
  const result = await answerPopup("\u001B");

  expect(result.status).toBe(0);
  expect(result.output).toContain('"answer":null');
});
