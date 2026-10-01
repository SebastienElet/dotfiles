# Local Fact Register

Use `~/.local/share/disaster-recovery-plan/<project>/context.yaml` as the default operational fact store.
Expand `~` to the user home and resolve relative source paths from this register's directory. It is not a replacement for code,
ADRs, contracts or policies. The example asset contains structure only; the runtime register holds
the answers. Populate a new register from the example without overwriting an existing one.

## Fact format

Every item in `facts` has these fields:

| Field                      | Meaning                                                                                                                                            |
| -------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `id`, `section`, `subject` | Stable identifier, template section, application/scenario/environment scope.                                                                       |
| `value`                    | A scalar or structured value; `null` means unknown, never an implicit negative. Include units for durations and specify calendar or business time. |
| `status`                   | `unknown`, `declared`, `verified`, `proposed`, `conflicting`, `stale` or `not_applicable`.                                                         |
| `kind`                     | `requirement`, `objective`, `configuration`, `procedure`, `test_result`, `approval` or `operational_fact`.                                         |
| `sources`                  | Source ID plus physical PDF page, DOCX section, repository commit/path/lines, user answer date, or execution report reference as appropriate.      |
| `observed_on`              | Date the evidence was read or answer received, not a fabricated test date.                                                                         |
| `owner`                    | Responsible role/person if known; otherwise `null`.                                                                                                |
| `validation`               | `by`, `on`, `evidence` and `environment`; leave unknown fields `null`.                                                                             |

`verified` means the stated fact has evidence at its own boundary. Reading a PAS verifies what it
declares, but does not verify the underlying configuration or recovery outcome: those facts remain
`declared`. Objectives approved by the business do not become measured recovery performance.

Source entries record `id`, `path` or `url`, `document_kind`, `scope`, `version`, `document_date`,
`approval_date`, `sha256`, `applicability` and `last_checked_on`. Source applicability starts as
`to_confirm` except for the supplied template's role as output structure.

For a conflicting fact, `value` is an object with `claims`: a list of objects containing `value`,
`sources` and `observed_on`. Keep the fact's `sources` as the union of those references. Do not
select a winning claim until the responsible authority or direct evidence resolves the conflict.
Each `history` entry contains `fact_id`, `changed_on`, `reason`, and `previous`: the complete prior
fact object, including provenance and validation. Append a history entry before changing an
existing fact's value or status. A resolution returns `value` to its ordinary scalar/object shape
and records the decision source; the previous conflicting fact remains in `history`.

## Update procedure

1. Parse the existing YAML with a native available YAML parser. If malformed, preserve it and
   report the parse error before overwriting anything. Never install a validator for this workflow.
2. Incorporate only accessible evidence or explicit user answers. Merge by stable fact ID and
   preserve unrelated entries. If a value changes, append the previous value and provenance to
   `history`; do not erase evidence of a disagreement.
3. If sources conflict, retain both claim values and their sources, set `status: conflicting`, and
   create an `open_points` entry. A newer file date alone does not establish authority.
4. Keep each open point's `id`, `section`, `question`, `owner`, `evidence_needed`, `blocks`,
   `status` and `resolution`. Close it only when the requested evidence or decision arrives.
5. Save to a temporary sibling, parse it, and replace the register only after successful parsing.
   Report the absolute register path and which facts changed. User answers are operational data:
   updating them does not require rewriting the skill.

Keep phone numbers and personal contact details in a controlled directory when appropriate and
store its location in the register. Never retain passwords, tokens, private keys or secret exports.

## Evidence companion

For a generated PRA, produce one Markdown companion with document status and scope, a table
mapping template sections to fact IDs and sources, conflicts/open points with owners, and the
generation checks including environment and unavailable visual verification. Keep unknowns visible
in the DOCX too. Separate agreed RTO/RPO, estimated procedure durations and measured test results.
