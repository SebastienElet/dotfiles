pub(super) fn message(code: &str, field: &str) -> &'static str {
    match code {
        "invalid_field" => field_requirement(field),
        "unsupported_schema" => "Use schema_version: 1; other schema versions are not supported.",
        "missing_proof" => "Provide at least one primary proof source in proof.sources.",
        "missing_oracle" => {
            "Provide oracle.automated with kind: source-fingerprint and expected: all-proof-sources-unchanged unless every source is a user-decision."
        }
        "too_many_items" => "Reduce the number of items to the published maximum.",
        "invalid_source_kind" => {
            "Use one source kind: git-file, local-file, official-url, or user-decision."
        }
        "duplicate_field" => {
            "Each YAML mapping key must occur once; remove duplicate keys near the reported location."
        }
        "malformed_yaml" => {
            "Provide one well-formed YAML mapping document; repair syntax near the reported location. No input excerpt is returned."
        }
        "unknown_field" => {
            "Remove fields not defined for this mapping in the entry contract; runtime-assigned fields do not belong in an admission draft."
        }
        "input_too_large" => {
            "Reduce stdin to at most 1048576 bytes (1 MiB), including serialization whitespace."
        }
        "invalid_utf8" => "Encode the complete stdin value as valid UTF-8.",
        "empty_stdin" => {
            "Provide the command's required document or text on stdin; the stream must not be empty."
        }
        "empty_query" => "Provide a query containing at least one non-whitespace character.",
        "sensitive_content" => {
            "Remove secrets, credentials, credential-bearing URLs, authorization headers, marked private prompts, and role-prefixed transcripts; retain only a safe durable summary."
        }
        "shell_command" => {
            "Replace executable shell text with a declarative durable statement; commands and command substitutions cannot be persisted."
        }
        "invalid_transition_reason" => {
            "Provide a reason containing a non-whitespace character and at most 500 Unicode scalar values."
        }
        "invalid_memory_id" => {
            "Use mem_ followed by exactly 24 lowercase hexadecimal digits, as returned by admit."
        }
        "entry_not_found" => {
            "No stored entry matches this id; use the id from the successful admission in the same configured memory store."
        }
        "invalid_human_conclusion" => {
            "Choose a compatible human status: goal achieved/abandoned, decision superseded, unknown resolved, assumption confirmed; evidence/invariant have none."
        }
        "entry_not_active" => {
            "Only an active entry can transition; terminal entries cannot be changed or reactivated."
        }
        "entry_not_terminal" => {
            "Supply a terminal status compatible with the entry kind; active is not a terminal status."
        }
        "entry_conflict" => {
            "This identity already has different canonical content; reconcile the accepted draft with the existing memory before another write."
        }
        "source_changed" => {
            "A proof source changed during admission; revalidate the proof and accepted draft before resubmitting."
        }
        "source_invalid" => {
            "Provide a supported, accessible primary source satisfying the source rules in the entry contract."
        }
        "source_unavailable" => {
            "The proof source could not be read or verified; preserve the draft and restore source access before resubmitting."
        }
        "scope_unavailable" => {
            "Project scope requires an accessible Git worktree with an unambiguous absolute canonical git-common-dir; run from that project. User scope requires separate explicit authorization."
        }
        "scope_mismatch" => {
            "The resolved project scope must match the entry scope; submit from the authorized project."
        }
        "admission_not_authorized" => {
            "Obtain an explicit persistence request or acceptance of the complete draft before admission."
        }
        "missing_hook_event" | "invalid_hook_event" => {
            "Set the JSON string hook_event_name to exactly UserPromptSubmit."
        }
        "missing_hook_query" | "invalid_hook_query" => {
            "Provide prompt as a JSON string containing at least one non-whitespace character."
        }
        "missing_hook_cwd" | "invalid_hook_cwd" => {
            "Provide cwd as an absolute JSON path string, nonempty and without NUL, parent traversal, or a leading dot component."
        }
        "invalid_hook_payload" => {
            "Provide one JSON object with string fields hook_event_name, prompt, cwd; do not repeat these keys."
        }
        "invalid_arguments" => {
            "Use admit --format json; retrieve --query-stdin --format json; confirm --id ID --status achieved|abandoned|superseded|resolved|confirmed --reason-stdin; hook --agent codex|claude; audit --format json [--include-terminal]. Do not repeat options or add positional arguments."
        }
        "invalid_kind_status" => {
            "Stored status must match kind: active/invalidated for all, achieved/abandoned for goal, superseded for decision, resolved for unknown, confirmed for assumption."
        }
        "unexpected_transition" => "An active stored entry must not contain a transition.",
        "missing_transition" => {
            "A terminal stored entry requires a transition recording its conclusion."
        }
        "invalid_transition" => {
            "A stored transition must start at active, end at the entry status, and use invalid for invalidated or valid for a human terminal status."
        }
        _ => availability_message(code),
    }
}

fn availability_message(code: &str) -> &'static str {
    match code {
        "stdin_unavailable" => {
            "The stdin stream could not be read; restore the input pipe and preserve the draft."
        }
        "output_unavailable" => {
            "The output stream could not be written or flushed; a write may already have succeeded. Restore the stream and reconcile before retrying."
        }
        "evaluation_trace_unavailable" => {
            "The optional evaluation trace is unavailable; check AGENT_MEMORY_EVAL_TRACE and AGENT_MEMORY_EVAL_AGENT configuration. A command may already have completed."
        }
        "retrieval_deadline_exceeded" => {
            "The hook's 25-second retrieval deadline expired; apply no memory and restore source availability before a later retrieval."
        }
        "oracle_unavailable" => {
            "The proof oracle is unavailable; apply no memory until sources can be verified or the required human confirmation is supplied."
        }
        "selection_stale" => "Memory changed after selection; retrieve again before consuming it.",
        "store_lock_timeout" => {
            "The store write lock timed out; wait for the competing writer to finish before retrying the accepted operation."
        }
        "store_permissions_unavailable" => {
            "Private store permissions could not be established (directories 0700, files 0600); preserve the draft and have the installation repaired."
        }
        "unsafe_store_path" => {
            "The store requires an absolute path, safe directory components, no symlinks and singly linked regular files; preserve the draft and have the installation repaired."
        }
        "memory_root_unavailable" => {
            "Configure an absolute AGENT_MEMORY_ROOT or a usable HOME for the default memory root."
        }
        "store_unavailable" | "store_lock_unavailable" | "cache_unavailable" => {
            "Local memory storage is unavailable; preserve the draft and have storage access repaired through supported tooling."
        }
        "entry_identity_mismatch" | "entry_path_mismatch" => {
            "Stored entry identity or scope does not match its canonical location; apply no memory and request store repair through supported tooling."
        }
        _ => {
            "The operation could not complete; preserve the draft and report this diagnostic code for runtime repair."
        }
    }
}

pub(crate) fn field_requirement(field: &str) -> &'static str {
    match field {
        "schema_version" => "Provide the required integer schema_version: 1.",
        "kind" => {
            "Provide kind as one of goal, decision, evidence, invariant, unknown, assumption."
        }
        "scope" => {
            "Use scope: project (the default) or scope: user with explicit user authorization."
        }
        "statement"
        | "proof.summary"
        | "oracle.human_fallback.question"
        | "oracle.human_fallback.valid_when"
        | "oracle.outcomes.valid"
        | "oracle.outcomes.invalidated"
        | "proof.sources.locator"
        | "transition.reason" => {
            "Provide the required string value; mappings, sequences and null are not accepted."
        }
        "retrieval_terms" => {
            "Provide a nonempty sequence of string retrieval terms (1 to 20 items, each 1 to 100 Unicode scalar values)."
        }
        "proof" => "Provide a proof mapping containing summary and sources.",
        "proof.sources" => {
            "Provide a sequence of 1 to 20 source mappings, each containing kind and locator."
        }
        "proof.sources.kind" => {
            "Provide source kind as git-file, local-file, official-url, or user-decision."
        }
        "oracle" => {
            "Provide an oracle mapping containing human_fallback and outcomes, and automated when required by the sources."
        }
        "oracle.automated" | "oracle.automated.kind" => {
            "Use an automated oracle mapping with kind: source-fingerprint and expected: all-proof-sources-unchanged."
        }
        "oracle.automated.expected" => {
            "Set the required string expected to all-proof-sources-unchanged."
        }
        "oracle.human_fallback" => {
            "Provide a human_fallback mapping containing question and valid_when strings."
        }
        "oracle.outcomes" => {
            "Provide an outcomes mapping containing valid and invalidated strings."
        }
        "id" => "Use mem_ followed by exactly 24 lowercase hexadecimal digits.",
        "scope.key" => "Use project_ followed by exactly 64 lowercase hexadecimal digits.",
        "proof.sources.fingerprint" => {
            "Use sha256: followed by exactly 64 lowercase hexadecimal digits."
        }
        "timestamp" => {
            "Use a real UTC calendar timestamp YYYY-MM-DDTHH:MM:SS[.fraction]Z, year 0001 or later, without leap seconds."
        }
        _ => {
            "Provide one YAML mapping matching the complete entry contract, with required fields, closed enum values and no additional fields."
        }
    }
}
