# Fee-call path

The distribution-key schema validates field shapes and accepts a caller-supplied `valid` boolean.
The adapter copies that boolean into the domain input.
`POST /fee-calls` is active and performs no relational lookup before posting.
