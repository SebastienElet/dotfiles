# Skill Activation Scenarios

## Purpose

`evals/trigger-queries.json` records realistic positive and negative activation prompts. The file is
a scenario format, not proof that any host executed the prompts.

This repository has no automatic multi-agent eval runner. Never claim activation rates or
cross-agent validation without actual recorded executions.

## Schema

```json
{
  "skill": "example-skill",
  "version": "1.0",
  "queries": [
    {
      "query": "Create an example artifact",
      "should_activate": true,
      "reason": "Direct request for the skill's workflow"
    },
    {
      "query": "Review an unrelated pull request",
      "should_activate": false,
      "reason": "A different skill owns pull request review"
    }
  ]
}
```

Doctor requires:

- string `skill` equal to the directory slug;
- string `version`;
- non-empty `queries` array;
- string `query`, boolean `should_activate`, and string `reason` per entry;
- at least one positive and one negative query.

## Writing scenarios

- Use phrases a user would actually type.
- Cover direct triggers and implicit needs that do not name the skill.
- Add adjacent-domain negatives that could false-positive.
- Explain the expected boundary in `reason`, not the implementation.
- Keep scenarios stable while comparing description or router changes.

## Running scenarios

Use fresh agent context so prior conclusions do not leak into the result. Give the agent the
repository and prompt, not the expected answer. Record whether the skill activated and whether its
procedure governed the response.

An eval file remains optional. Create one when activation behavior is critical or a description is
being changed for routing reasons.

## Activation router evidence

Descriptions route by default. Before adding an external router rule:

1. Select the relevant positive and adjacent negative queries.
2. Run each query at least three times without the proposed rule.
3. Require a repeated missed or wrong activation; one anomalous run is insufficient.
4. Record the baseline prompts and outcomes.
5. Add the smallest relational rule that distinguishes the competing skills.
6. Run the identical prompts again and compare outcomes.
7. Remove the rule if it does not materially improve routing.

The absence of a router is never itself a finding.

## Constraints

- Never present scenario files as executed evidence.
- Never invent activation metrics.
- Never add a router from one run or intuition alone.
- Never change prompts between baseline and comparison.
- Always include positive and negative cases in a present eval file.
