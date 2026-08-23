use super::{Agent, Manifest, Scope};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CommandDeclaration {
    pub(super) name: String,
    pub(super) description: String,
    pub(super) prompt: String,
    pub(super) bindings: Vec<CommandBindingDeclaration>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CommandBindingDeclaration {
    pub(super) agent: Agent,
    pub(super) scope: Scope,
}

#[derive(Clone, Copy)]
pub struct Command<'a> {
    declaration: &'a CommandDeclaration,
}

#[derive(Clone, Copy)]
pub struct CommandBinding<'a> {
    command: &'a CommandDeclaration,
    pub agent: Agent,
    pub scope: Scope,
}

impl Manifest {
    pub fn commands(&self) -> impl Iterator<Item = Command<'_>> {
        self.commands.iter().map(Command::from)
    }
}

impl<'a> From<&'a CommandDeclaration> for Command<'a> {
    fn from(declaration: &'a CommandDeclaration) -> Self {
        Self { declaration }
    }
}

impl<'a> Command<'a> {
    pub fn name(self) -> &'a str {
        &self.declaration.name
    }

    pub fn description(self) -> &'a str {
        &self.declaration.description
    }

    pub fn prompt(self) -> &'a str {
        &self.declaration.prompt
    }

    pub fn bindings(self) -> impl ExactSizeIterator<Item = CommandBinding<'a>> {
        self.declaration
            .bindings
            .iter()
            .map(move |binding| CommandBinding {
                command: self.declaration,
                agent: binding.agent,
                scope: binding.scope,
            })
    }
}

impl<'a> CommandBinding<'a> {
    pub fn name(self) -> &'a str {
        &self.command.name
    }

    pub fn description(self) -> &'a str {
        &self.command.description
    }

    pub fn prompt(self) -> &'a str {
        &self.command.prompt
    }
}
