use std::collections::HashSet;

use crate::Identity;
use crate::proto::v1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resource {
    UserConfig(UserConfigResource),
    Dynamic(DynamicResource),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceMeta {
    id: Identity,
    children: HashSet<Identity>,
    spec: Vec<u8>,
    status: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserConfigResource {
    meta: ResourceMeta,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicResource {
    meta: ResourceMeta,
    owner: Identity,
    dependencies: HashSet<Identity>,
}

impl Resource {
    pub fn meta(&self) -> &ResourceMeta {
        match self {
            Resource::UserConfig(res) => res.meta(),
            Resource::Dynamic(res) => res.meta(),
        }
    }

    pub fn meta_mut(&mut self) -> &mut ResourceMeta {
        match self {
            Resource::UserConfig(res) => res.meta_mut(),
            Resource::Dynamic(res) => res.meta_mut(),
        }
    }

    pub fn maybe_user_config(&self) -> Option<&UserConfigResource> {
        match self {
            Resource::UserConfig(res) => Some(res),
            _ => None,
        }
    }

    pub fn maybe_user_config_mut(&mut self) -> Option<&mut UserConfigResource> {
        match self {
            Resource::UserConfig(res) => Some(res),
            _ => None,
        }
    }

    pub fn maybe_dynamic(&self) -> Option<&DynamicResource> {
        match self {
            Resource::Dynamic(res) => Some(res),
            _ => None,
        }
    }

    pub fn maybe_dynamic_mut(&mut self) -> Option<&mut DynamicResource> {
        match self {
            Resource::Dynamic(res) => Some(res),
            _ => None,
        }
    }
}

impl ResourceMeta {
    pub fn new(id: Identity, spec: Vec<u8>) -> Self {
        Self {
            id,
            children: HashSet::default(),
            spec,
            status: Vec::default(),
        }
    }

    pub fn id(&self) -> &Identity {
        &self.id
    }

    pub fn children(&self) -> &HashSet<Identity> {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut HashSet<Identity> {
        &mut self.children
    }

    pub fn spec(&self) -> &[u8] {
        &self.spec
    }

    pub fn spec_mut(&mut self) -> &mut Vec<u8> {
        &mut self.spec
    }

    pub fn status(&self) -> &[u8] {
        &self.status
    }

    pub fn status_mut(&mut self) -> &mut Vec<u8> {
        &mut self.status
    }
}

impl UserConfigResource {
    pub fn new(meta: ResourceMeta) -> Self {
        Self { meta }
    }

    pub fn meta(&self) -> &ResourceMeta {
        &self.meta
    }

    pub fn meta_mut(&mut self) -> &mut ResourceMeta {
        &mut self.meta
    }
}

impl DynamicResource {
    pub fn new(meta: ResourceMeta, owner: Identity) -> Self {
        Self {
            meta,
            owner,
            dependencies: HashSet::default(),
        }
    }

    pub fn meta(&self) -> &ResourceMeta {
        &self.meta
    }

    pub fn meta_mut(&mut self) -> &mut ResourceMeta {
        &mut self.meta
    }

    pub fn owner(&self) -> &Identity {
        &self.owner
    }

    pub fn dependencies(&self) -> &HashSet<Identity> {
        &self.dependencies
    }

    pub fn dependencies_mut(&mut self) -> &mut HashSet<Identity> {
        &mut self.dependencies
    }
}

impl From<UserConfigResource> for Resource {
    fn from(value: UserConfigResource) -> Self {
        Self::UserConfig(value)
    }
}

impl From<DynamicResource> for Resource {
    fn from(value: DynamicResource) -> Self {
        Self::Dynamic(value)
    }
}

impl From<ResourceMeta> for UserConfigResource {
    fn from(value: ResourceMeta) -> Self {
        Self::new(value)
    }
}

impl From<ResourceMeta> for v1::ResourceMeta {
    fn from(value: ResourceMeta) -> Self {
        Self {
            id: Some(value.id.into()),
            children: value.children.into_iter().map(From::from).collect(),
            spec: value.spec,
            status: value.status,
        }
    }
}

impl From<Resource> for v1::MetaResource {
    fn from(value: Resource) -> Self {
        Self {
            resource_type: Some(value.into()),
        }
    }
}

impl From<Resource> for v1::meta_resource::ResourceType {
    fn from(value: Resource) -> Self {
        match value {
            Resource::UserConfig(res) => Self::UserConfig(res.into()),
            Resource::Dynamic(res) => Self::Dynamic(res.into()),
        }
    }
}

impl From<UserConfigResource> for v1::UserConfigResource {
    fn from(value: UserConfigResource) -> Self {
        Self {
            meta: Some(value.meta.into()),
        }
    }
}

impl From<DynamicResource> for v1::DynamicResource {
    fn from(value: DynamicResource) -> Self {
        Self {
            meta: Some(value.meta.into()),
            owner: Some(value.owner.into()),
            dependencies: value
                .dependencies
                .into_iter()
                .map(From::from)
                .collect(),
            dependents: vec![],
        }
    }
}
