//! # ActorID
//! Contains a basic [`ActorID`] type that represents only foreign actors.

use fluxion::Identifier;



/// # [`ActorID`]
/// This enum is used to identify an actor that is on a specific foreign system.
/// This is used instead of [`Identifier`] in situations where the actor is known to be on a foreign system,
/// so local variants are not required
pub enum ActorID {
    /// # [`ActorID::Foreign`]
    /// Represents a foreign actor on the given system with a numeric ID.
    Foreign {
        /// The foreign actor's numeric id
        id: u64,
        /// The name of the foreign system
        system: String,
    },
    /// # [`ActorID::ForeignNamed`]
    /// Represents a foreign actor on the given system with a string ID (name)
    ForeignNamed {
        /// The foreign actor's name
        name: String,
        /// The name of the foreign system
        system: String,
    }
}

impl ActorID {
    /// # [`ActorID::from_identifier`]
    /// Creates a new [`ActorID`] from the given [`Identifier`].
    /// Returns [`None`] if the [`Identifier`] represents a local actor.
    pub fn from_identifier(identifier: Identifier) -> Option<Self> {
        match identifier {
            Identifier::Foreign(id, system) => Some(Self::Foreign { id, system: system.to_string() }),
            Identifier::ForeignNamed(name, system) => Some(Self::ForeignNamed { name: name.to_string(), system: system.to_string() }),
            _ => None,
        }
    } 
}