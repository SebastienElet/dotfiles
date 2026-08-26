import { appendFileSync, mkdirSync } from "node:fs";
import { z } from "zod";

const environmentSchema = z.object({
  PAID_APPS_TEST_DESTINATION: z.string().min(1),
  PAID_APPS_TEST_TRACE: z.string().min(1),
});
const argumentsSchema = z.tuple([z.literal("install"), z.string().min(1)]);
const providerArgumentOffset = 2;

const environment = environmentSchema.parse(process.env);
const providerArguments = argumentsSchema.parse(
  process.argv.slice(providerArgumentOffset),
);

appendFileSync(
  environment.PAID_APPS_TEST_TRACE,
  `${providerArguments.join(" ")}\n`,
);
mkdirSync(environment.PAID_APPS_TEST_DESTINATION, { recursive: true });
