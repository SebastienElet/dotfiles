function recoveryExperimentEvidence(profile: string): string {
  const windows = [
    "window-amber-11",
    "window-cobalt-19",
    "window-fern-23",
    "window-ivory-29",
    "window-mica-37",
    "window-onyx-41",
    "window-pearl-43",
    "window-quartz-47",
    "window-slate-53",
  ];
  const seeds = [
    "seed-ash-13",
    "seed-birch-17",
    "seed-cedar-31",
    "seed-ember-91",
    "seed-fir-101",
    "seed-hazel-127",
    "seed-maple-149",
    "seed-oak-163",
    "seed-yew-181",
  ];
  let trial = 0;
  const results = windows.flatMap((window) =>
    seeds.map((seed) => {
      trial += 1;
      const accepted = window === "window-mica-37" && seed === "seed-ember-91";
      return `${window},${seed},${accepted ? "accepted" : "rejected"},${accepted ? profile : `profile-${trial}`}`;
    }),
  );
  return [
    "transport_window,checksum_seed,outcome,controller_profile",
    ...results,
    "",
  ].join("\n");
}

export { recoveryExperimentEvidence };
