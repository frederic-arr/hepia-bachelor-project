use cos_api_internal_server::proto::v1;
use cos_api_shared::Identity;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CreateConfig {
    pub id: Identity,
    pub spec: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CreateResource {
    pub id: Identity,
    pub owner: Identity,
    pub spec: Vec<u8>,
}

// #[derive(Debug, Clone, Default, PartialEq, Eq)]
// pub struct StoredResource {
//     /// The unique identity of the resource.
//     pub id: Identity,

//     /// Owner of the current resource. If the owner of the resource is makred
//     /// for deletion, the current resource will be marked for deletion as
//     /// well.
//     pub owner: Owner,

//     /// List of identities of resources directly created by the current
//     /// resource.
//     pub children: HashSet<Identity>,

//     /// List of resources for which deletion will be blocked until the
// current     /// resource is deleted or the dependency is removed.
//     pub dependencies: HashSet<Identity>,

//     /// List of resources that depends on the current resource and which will
//     /// block its deletion.
//     pub dependents: HashSet<Identity>,

//     /// Desired state
//     pub spec: Vec<u8>,

//     /// Actual state
//     pub status: Vec<u8>,
// }

// impl Display for Identity {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.write_str(&self.schema)?;
//         f.write_char('/')?;
//         f.write_str(&self.name)
//     }
// }

// impl Default for Owner {
//     fn default() -> Self {
//         Self::Config(Identity::default())
//     }
// }

// impl From<api_internal_server::Identity> for Identity {
//     fn from(value: api_internal_server::Identity) -> Self {
//         Self {
//             name: value.name,
//             schema: value.schema,
//         }
//     }
// }

// impl From<Option<api_internal_server::Identity>> for Identity {
//     fn from(value: Option<api_internal_server::Identity>) -> Self {
//         value.unwrap_or_default().into()
//     }
// }

// impl From<Option<&api_internal_server::Identity>> for Identity {
//     fn from(value: Option<&api_internal_server::Identity>) -> Self {
//         value.cloned().unwrap_or_default().into()
//     }
// }

// impl From<&Option<api_internal_server::Identity>> for Identity {
//     fn from(value: &Option<api_internal_server::Identity>) -> Self {
//         value.clone().unwrap_or_default().into()
//     }
// }

// impl From<Identity> for api_internal_server::Identity {
//     fn from(value: Identity) -> Self {
//         Self {
//             name: value.name,
//             schema: value.schema,
//         }
//     }
// }

impl TryFrom<v1::ResourceCreateDynamicRequest> for CreateResource {
    type Error = ();

    fn try_from(
        value: v1::ResourceCreateDynamicRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.try_into()?,
            owner: value.owner.try_into()?,
            spec: value.spec,
        })
    }
}

// impl From<CreateResource> for StoredResource {
//     fn from(value: CreateResource) -> Self {
//         Self {
//             id: value.id,
//             owner: value.owner,
//             spec: value.spec,
//             ..Default::default()
//         }
//     }
// }

// impl From<CreateConfig> for StoredConfig {
//     fn from(value: CreateConfig) -> Self {
//         Self {
//             id: value.id,
//             spec: value.spec,
//             ..Default::default()
//         }
//     }
// }

// impl Owner {
//     #[must_use]
//     pub const fn identity(&self) -> &Identity {
//         match self {
//             Self::Config(id) | Self::Resource(id) => id,
//         }
//     }
// }

// impl From<Owner> for (api_internal_server::OwnerType, Identity) {
//     fn from(value: Owner) -> Self {
//         match value {
//             Owner::Config(id) => (api_internal_server::OwnerType::Config,
// id),             Owner::Resource(id) => {
//                 (api_internal_server::OwnerType::Resource, id)
//             }
//         }
//     }
// }
