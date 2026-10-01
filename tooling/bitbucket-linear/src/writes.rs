use crate::{
    core::AttachmentAction,
    linear::{validate_attachment, validate_state},
    model::{Attachment, Fallible, IssueId, State, StateId, StateType},
    providers::{Providers, parse},
};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct AttachmentResult {
    success: bool,
    attachment: Attachment,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttachmentData {
    attachment_create: AttachmentResult,
}
#[derive(Deserialize)]
struct UpdatedIssue {
    id: IssueId,
    state: State,
}
#[derive(Deserialize)]
struct CompletionResult {
    success: bool,
    issue: UpdatedIssue,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompletionData {
    issue_update: CompletionResult,
}

pub fn attach(providers: &Providers<'_>, id: IssueId, action: &AttachmentAction) -> Fallible<()> {
    let data: AttachmentData = parse(providers.graphql("mutation Attach($input: AttachmentCreateInput!) { attachmentCreate(input:$input) { success attachment { id url title subtitle } } }", &json!({"input":{"issueId":id,"url":action.url,"title":action.title,"subtitle":action.subtitle}}))?)?;
    let actual = data.attachment_create.attachment;
    validate_attachment(&actual)?;
    if !data.attachment_create.success
        || actual.url != action.url
        || actual.title != action.title
        || actual.subtitle.as_deref() != Some(action.subtitle.as_str())
    {
        return Err("attachment mutation mismatch");
    }
    let stored = providers.issue(&id.to_string())?;
    let matches: Vec<_> = stored
        .attachments
        .iter()
        .filter(|a| a.url == action.url)
        .collect();
    if matches.len() != 1
        || matches.first().is_none_or(|a| {
            a.title != action.title || a.subtitle.as_deref() != Some(action.subtitle.as_str())
        })
    {
        return Err("attachment readback mismatch");
    }
    Ok(())
}

pub fn complete(providers: &Providers<'_>, id: IssueId, state_id: StateId) -> Fallible<()> {
    let data: CompletionData = parse(providers.graphql("mutation Complete($id:String!, $input:IssueUpdateInput!) { issueUpdate(id:$id,input:$input) { success issue { id state { id name type } } } }", &json!({"id":id,"input":{"stateId":state_id}}))?)?;
    let updated = data.issue_update.issue;
    validate_state(&updated.state)?;
    if !data.issue_update.success
        || updated.id != id
        || updated.state.id != state_id
        || updated.state.kind != StateType::Completed
    {
        return Err("completion mutation mismatch");
    }
    let stored = providers.issue(&id.to_string())?;
    if stored.state.id != state_id || stored.state.kind != StateType::Completed {
        return Err("completion readback mismatch");
    }
    Ok(())
}
