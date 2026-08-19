import { branchMainValue } from "./branch.js";
import { liveValue } from "./live.js";
import { removableValue } from "./removable.js";

export const entryValue = branchMainValue + liveValue + removableValue;
