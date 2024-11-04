//! # ActorID
//! Contains a basic [`ActorID`] type that represents actors without any regard to the system.

use fluxion::Identifier;



/// # [`ActorID`]
/// This enum is used to identify an actor in contexts where the system doesn't matter.
/// This is used instead of [`Identifier`] in situations where the actor's location is already known.
#[derive(PartialEq, Eq, Hash)]
pub enum ActorID {
    /// # [`ActorID::`]
    /// Represents an actor with a numeric ID.
    Numeric(u64),
    /// # [`ActorID::Named`]
    /// Represents an actor with a string ID (name)
    Named(String)
}

impl From<Identifier<'_>> for ActorID {
    fn from(value: Identifier) -> Self {
        match value {
            Identifier::Local(id) => Self::Numeric(id),
            Identifier::LocalNamed(name) => Self::Named(name.to_string()),
            Identifier::Foreign(id, _) => Self::Numeric(id),
            Identifier::ForeignNamed(name, _) => Self::Named(name.to_string()),
        }
    }
}