ObjC.import("ServiceManagement");

function run(argv) {
  if (argv.length !== 1 || typeof argv[0] !== "string" || argv[0].length === 0) {
    throw new Error("A service label is required");
  }
  const raw = $.SMCopyAllJobDictionaries($.kSMDomainUserLaunchd);
  const jobs = ObjC.deepUnwrap(ObjC.castRefToObject(raw));
  if (!Array.isArray(jobs)) {
    throw new Error("Native service snapshot is unavailable or has an unknown shape");
  }
  for (const job of jobs) {
    if (job === null || typeof job !== "object" || Array.isArray(job) || typeof job.Label !== "string" || job.Label.length === 0) {
      throw new Error("Native service snapshot contains an unknown job");
    }
  }
  const selected = jobs.filter(function (job) { return job.Label === argv[0]; });
  if (selected.length > 1) {
    throw new Error("Native service snapshot contains an ambiguous label");
  }
  const job = selected[0];
  if (job === undefined) {
    return JSON.stringify({ state: "absent" });
  }
  if (!Array.isArray(job.ProgramArguments) || job.ProgramArguments.length === 0 || job.ProgramArguments.some(function (value) { return typeof value !== "string" || value.length === 0; }) || (job.Program !== undefined && (typeof job.Program !== "string" || job.Program.length === 0))) {
    throw new Error("Native service definition has an unknown shape");
  }
  return JSON.stringify({ state: "loaded", definition: { Label: job.Label, Program: job.Program, ProgramArguments: job.ProgramArguments } });
}
