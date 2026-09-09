use super::{
    contracts::{Observation, Oracle, Tool},
    report::Status,
};

pub fn evaluate(oracle: Oracle, observations: &[Observation]) -> Status {
    let read_index = observations.iter().position(|event| {
        event.tool == Tool::Cat
            && event.exit_code == 0
            && event
                .args
                .iter()
                .any(|arg| arg == ".agents/skills/code-search/SKILL.md")
    });
    let conceptual_index = observations
        .iter()
        .position(|event| event.tool == Tool::ColgrepSearch && event.exit_code == 0);
    let passes = match oracle {
        Oracle::StructuralV1 => {
            matches!((read_index, conceptual_index), (Some(read), Some(search)) if search > read)
        }
        Oracle::LiteralV1 => {
            !observations
                .iter()
                .any(|event| event.tool == Tool::ColgrepSearch)
                && observations.iter().any(|event| {
                    event.tool == Tool::Rg
                        && event.exit_code == 0
                        && event.args.iter().any(|arg| arg == "FEATURE_FLAG_DISABLED")
                })
        }
        Oracle::KnownPathV1 => {
            !observations
                .iter()
                .any(|event| matches!(event.tool, Tool::ColgrepSearch | Tool::Rg | Tool::Fd))
                && observations.iter().any(|event| {
                    event.tool == Tool::Cat
                        && event.exit_code == 0
                        && event.args.iter().any(|arg| arg == "src/auth/session.ts")
                })
        }
    };
    if passes { Status::Pass } else { Status::Fail }
}
