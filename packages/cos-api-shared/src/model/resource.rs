use std::collections::HashSet;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::Identity;
use crate::proto::v1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resource<Spec = rmpv::Value, Status = rmpv::Value> {
    UserConfig(UserConfigResource<Spec, Status>),
    Dynamic(DynamicResource<Spec, Status>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceMeta<Spec = rmpv::Value, Status = rmpv::Value> {
    id: Identity,
    children: HashSet<Identity>,
    spec: Spec,
    status: Option<Status>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserConfigResource<Spec = rmpv::Value, Status = rmpv::Value> {
    meta: ResourceMeta<Spec, Status>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicResource<Spec = rmpv::Value, Status = rmpv::Value> {
    meta: ResourceMeta<Spec, Status>,
    owner: Identity,
    dependencies: HashSet<Identity>,
}

impl<Spec, Status> Resource<Spec, Status> {
    pub fn meta(&self) -> &ResourceMeta<Spec, Status> {
        match self {
            Resource::UserConfig(res) => res.meta(),
            Resource::Dynamic(res) => res.meta(),
        }
    }

    pub fn meta_mut(&mut self) -> &mut ResourceMeta<Spec, Status> {
        match self {
            Resource::UserConfig(res) => res.meta_mut(),
            Resource::Dynamic(res) => res.meta_mut(),
        }
    }

    pub fn maybe_user_config(
        &self,
    ) -> Option<&UserConfigResource<Spec, Status>> {
        match self {
            Resource::UserConfig(res) => Some(res),
            _ => None,
        }
    }

    pub fn maybe_user_config_mut(
        &mut self,
    ) -> Option<&mut UserConfigResource<Spec, Status>> {
        match self {
            Resource::UserConfig(res) => Some(res),
            _ => None,
        }
    }

    pub fn maybe_dynamic(&self) -> Option<&DynamicResource<Spec, Status>> {
        match self {
            Resource::Dynamic(res) => Some(res),
            _ => None,
        }
    }

    pub fn maybe_dynamic_mut(
        &mut self,
    ) -> Option<&mut DynamicResource<Spec, Status>> {
        match self {
            Resource::Dynamic(res) => Some(res),
            _ => None,
        }
    }

    pub fn id(&self) -> &Identity {
        &self.meta().id()
    }

    pub fn children(&self) -> &HashSet<Identity> {
        &self.meta().children()
    }

    pub fn children_mut(&mut self) -> &mut HashSet<Identity> {
        let meta = self.meta_mut();
        meta.children_mut()
    }

    pub fn spec(&self) -> &Spec {
        &self.meta().spec()
    }

    pub fn spec_mut(&mut self) -> &mut Spec {
        let meta = self.meta_mut();
        meta.spec_mut()
    }

    pub fn status(&self) -> Option<&Status> {
        self.meta().status()
    }

    pub fn status_mut(&mut self) -> Option<&mut Status> {
        let meta = self.meta_mut();
        meta.status_mut()
    }

    pub fn status_opt(&self) -> &Option<Status> {
        self.meta().status_opt()
    }

    pub fn status_opt_mut(&mut self) -> &mut Option<Status> {
        self.meta_mut().status_opt_mut()
    }
}

impl<Spec, Status> ResourceMeta<Spec, Status> {
    pub fn new(id: Identity, spec: Spec) -> Self {
        Self {
            id,
            children: HashSet::default(),
            spec,
            status: None,
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

    pub fn spec(&self) -> &Spec {
        &self.spec
    }

    pub fn spec_mut(&mut self) -> &mut Spec {
        &mut self.spec
    }

    pub fn status(&self) -> Option<&Status> {
        self.status.as_ref()
    }

    pub fn status_mut(&mut self) -> Option<&mut Status> {
        self.status.as_mut()
    }

    pub fn status_opt(&self) -> &Option<Status> {
        &self.status
    }

    pub fn status_opt_mut(&mut self) -> &mut Option<Status> {
        &mut self.status
    }
}

impl<Spec, Status> UserConfigResource<Spec, Status> {
    pub fn new(meta: ResourceMeta<Spec, Status>) -> Self {
        Self { meta }
    }

    pub fn meta(&self) -> &ResourceMeta<Spec, Status> {
        &self.meta
    }

    pub fn meta_mut(&mut self) -> &mut ResourceMeta<Spec, Status> {
        &mut self.meta
    }
}

impl<Spec, Status> DynamicResource<Spec, Status> {
    pub fn new(meta: ResourceMeta<Spec, Status>, owner: Identity) -> Self {
        Self {
            meta,
            owner,
            dependencies: HashSet::default(),
        }
    }

    pub fn meta(&self) -> &ResourceMeta<Spec, Status> {
        &self.meta
    }

    pub fn meta_mut(&mut self) -> &mut ResourceMeta<Spec, Status> {
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

impl<Spec, Status> TryFrom<ResourceMeta<Spec, Status>> for v1::ResourceMeta
where
    Spec: Serialize,
    Status: Serialize,
{
    type Error = String;

    fn try_from(
        value: ResourceMeta<Spec, Status>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Some(value.id.into()),
            children: value.children.into_iter().map(From::from).collect(),
            spec: rmp_serde::to_vec(&value.spec)
                .map_err(|_| "invalid spec".to_string())?,
            status: rmp_serde::to_vec(&value.status)
                .map_err(|_| "invalid status".to_string())?,
        })
    }
}

impl<Spec, Status> TryFrom<Resource<Spec, Status>> for v1::MetaResource
where
    Spec: Serialize,
    Status: Serialize,
{
    type Error = String;

    fn try_from(value: Resource<Spec, Status>) -> Result<Self, Self::Error> {
        Ok(Self {
            resource_type: Some(value.try_into()?),
        })
    }
}

impl<Spec, Status> TryFrom<Resource<Spec, Status>>
    for v1::meta_resource::ResourceType
where
    Spec: Serialize,
    Status: Serialize,
{
    type Error = String;

    fn try_from(value: Resource<Spec, Status>) -> Result<Self, Self::Error> {
        Ok(match value {
            Resource::UserConfig(res) => Self::UserConfig(res.try_into()?),
            Resource::Dynamic(res) => Self::Dynamic(res.try_into()?),
        })
    }
}

impl<Spec, Status> TryFrom<UserConfigResource<Spec, Status>>
    for v1::UserConfigResource
where
    Spec: Serialize,
    Status: Serialize,
{
    type Error = String;

    fn try_from(
        value: UserConfigResource<Spec, Status>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            meta: Some(value.meta.try_into()?),
        })
    }
}

impl<Spec, Status> TryFrom<DynamicResource<Spec, Status>>
    for v1::DynamicResource
where
    Spec: Serialize,
    Status: Serialize,
{
    type Error = String;

    fn try_from(
        value: DynamicResource<Spec, Status>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            meta: Some(value.meta.try_into()?),
            owner: Some(value.owner.into()),
            dependencies: value
                .dependencies
                .into_iter()
                .map(From::from)
                .collect(),
            dependents: vec![],
        })
    }
}

impl<Spec, Status> TryFrom<v1::ResourceMeta> for ResourceMeta<Spec, Status>
where
    Spec: DeserializeOwned,
    Status: DeserializeOwned,
{
    type Error = String;

    fn try_from(value: v1::ResourceMeta) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.try_into()?,
            children: value
                .children
                .into_iter()
                .map(Identity::try_from)
                .try_collect()?,
            spec: rmp_serde::from_slice(&value.spec)
                .map_err(|_| "invalid spec".to_string())?,
            status: if value.status.is_empty() {
                None
            } else {
                rmp_serde::from_slice::<Status>(&value.status)
                    .map(Some)
                    .map_err(|_| "invalid status".to_string())?
            },
        })
    }
}

impl<Spec, Status> TryFrom<Option<v1::ResourceMeta>>
    for ResourceMeta<Spec, Status>
where
    Spec: DeserializeOwned,
    Status: DeserializeOwned,
{
    type Error = String;

    fn try_from(value: Option<v1::ResourceMeta>) -> Result<Self, Self::Error> {
        value
            .ok_or("ResourceMeta is required".to_string())?
            .try_into()
    }
}

impl<Spec, Status> TryFrom<v1::MetaResource> for Resource<Spec, Status>
where
    Spec: DeserializeOwned,
    Status: DeserializeOwned,
{
    type Error = String;

    fn try_from(value: v1::MetaResource) -> Result<Self, Self::Error> {
        match value
            .resource_type
            .ok_or("ResourceType is required".to_string())?
        {
            v1::meta_resource::ResourceType::UserConfig(res) => {
                res.try_into().map(Self::UserConfig)
            }
            v1::meta_resource::ResourceType::Dynamic(res) => {
                res.try_into().map(Self::Dynamic)
            }
        }
    }
}

impl<Spec, Status> TryFrom<Option<v1::MetaResource>> for Resource<Spec, Status>
where
    Spec: DeserializeOwned,
    Status: DeserializeOwned,
{
    type Error = String;

    fn try_from(value: Option<v1::MetaResource>) -> Result<Self, Self::Error> {
        value
            .ok_or("MetaResource is required".to_string())?
            .try_into()
    }
}

impl<Spec, Status> TryFrom<v1::UserConfigResource>
    for UserConfigResource<Spec, Status>
where
    Spec: DeserializeOwned,
    Status: DeserializeOwned,
{
    type Error = String;

    fn try_from(value: v1::UserConfigResource) -> Result<Self, Self::Error> {
        Ok(Self {
            meta: value.meta.try_into()?,
        })
    }
}

impl<Spec, Status> TryFrom<Option<v1::UserConfigResource>>
    for UserConfigResource<Spec, Status>
where
    Spec: DeserializeOwned,
    Status: DeserializeOwned,
{
    type Error = String;

    fn try_from(
        value: Option<v1::UserConfigResource>,
    ) -> Result<Self, Self::Error> {
        value
            .ok_or("UserConfigResource is required".to_string())?
            .try_into()
    }
}

impl<Spec, Status> TryFrom<v1::DynamicResource>
    for DynamicResource<Spec, Status>
where
    Spec: DeserializeOwned,
    Status: DeserializeOwned,
{
    type Error = String;

    fn try_from(value: v1::DynamicResource) -> Result<Self, Self::Error> {
        Ok(Self {
            meta: value.meta.try_into()?,
            owner: value.owner.try_into()?,
            dependencies: value
                .dependencies
                .into_iter()
                .map(Identity::try_from)
                .try_collect()?,
        })
    }
}

impl<Spec, Status> TryFrom<Option<v1::DynamicResource>>
    for DynamicResource<Spec, Status>
where
    Spec: DeserializeOwned,
    Status: DeserializeOwned,
{
    type Error = String;

    fn try_from(
        value: Option<v1::DynamicResource>,
    ) -> Result<Self, Self::Error> {
        value
            .ok_or("DynamicResource is required".to_string())?
            .try_into()
    }
}
