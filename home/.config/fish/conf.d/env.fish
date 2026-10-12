# Setup env
if not string match -rq '\S' -- "$RLM_SESSION_DIR"
    if test -n "$PRIME_AGENT_SESSION_DIR"
        set -gx RLM_SESSION_DIR "$PRIME_AGENT_SESSION_DIR"
    else if not set -q PRIME_AGENT_SESSION_DIR; and test -n "$PRIME_AGENT_CODING_AGENT_SESSION_DIR"
        set -gx RLM_SESSION_DIR "$PRIME_AGENT_CODING_AGENT_SESSION_DIR"
    else if test -n "$PRIME_AGENT_CODING_AGENT_DIR"
        set -gx RLM_SESSION_DIR "$PRIME_AGENT_CODING_AGENT_DIR/sessions"
    else
        set -gx RLM_SESSION_DIR "$HOME/.prime/agent/sessions"
    end
end

if not string match -rq '\S' -- "$RLM_HARNESS_STATE_DIR"
    set -gx RLM_HARNESS_STATE_DIR "$RLM_SESSION_DIR/harness"
end
