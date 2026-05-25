use std::collections::HashSet;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::proto::v1;
use crate::{Identity, Specification, State};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize",
    deserialize = "T: DeserializeOwned"
))]
pub enum Resource<T>
where
    T: Specification,
{
    UserConfig(UserConfigResource<T>),
    Dynamic(DynamicResource<T>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize",
    deserialize = "T: DeserializeOwned"
))]
pub struct ResourceMeta<T>
where
    T: Specification,
{
    id: Identity,
    children: HashSet<Identity>,
    spec: T,
    state: Option<T::State>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize",
    deserialize = "T: DeserializeOwned"
))]
pub struct UserConfigResource<T>
where
    T: Specification,
{
    meta: ResourceMeta<T>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize",
    deserialize = "T: DeserializeOwned"
))]
pub struct DynamicResource<T>
where
    T: Specification,
{
    meta: ResourceMeta<T>,
    owner: Identity,
    dependencies: HashSet<Identity>,
}

impl<T> Resource<T>
where
    T: Specification,
{
    pub const fn meta(&self) -> &ResourceMeta<T> {
        match self {
            Self::UserConfig(res) => res.meta(),
            Self::Dynamic(res) => res.meta(),
        }
    }

    pub const fn meta_mut(&mut self) -> &mut ResourceMeta<T> {
        match self {
            Self::UserConfig(res) => res.meta_mut(),
            Self::Dynamic(res) => res.meta_mut(),
        }
    }

    pub const fn maybe_user_config(&self) -> Option<&UserConfigResource<T>> {
        match self {
            Self::UserConfig(res) => Some(res),
            Self::Dynamic(_) => None,
        }
    }

    pub const fn maybe_user_config_mut(
        &mut self,
    ) -> Option<&mut UserConfigResource<T>> {
        match self {
            Self::UserConfig(res) => Some(res),
            Self::Dynamic(_) => None,
        }
    }

    pub const fn maybe_dynamic(&self) -> Option<&DynamicResource<T>> {
        match self {
            Self::Dynamic(res) => Some(res),
            Self::UserConfig(_) => None,
        }
    }

    pub const fn maybe_dynamic_mut(&mut self) -> Option<&mut DynamicResource<T>> {
        match self {
            Self::Dynamic(res) => Some(res),
            Self::UserConfig(_) => None,
        }
    }

    pub const fn id(&self) -> &Identity {
        self.meta().id()
    }

    pub const fn children(&self) -> &HashSet<Identity> {
        self.meta().children()
    }

    pub const fn children_mut(&mut self) -> &mut HashSet<Identity> {
        let meta = self.meta_mut();
        meta.children_mut()
    }

    pub const fn spec(&self) -> &T {
        self.meta().spec()
    }

    pub const fn spec_mut(&mut self) -> &mut T {
        let meta = self.meta_mut();
        meta.spec_mut()
    }

    pub const fn state(&self) -> Option<&T::State> {
        self.meta().state()
    }

    pub const fn state_mut(&mut self) -> Option<&mut T::State> {
        let meta = self.meta_mut();
        meta.state_mut()
    }

    pub const fn state_opt(&self) -> &Option<T::State> {
        self.meta().state_opt()
    }

    pub const fn state_opt_mut(&mut self) -> &mut Option<T::State> {
        self.meta_mut().state_opt_mut()
    }
}

impl<T> ResourceMeta<T>
where
    T: Specification,
{
    pub fn new(id: Identity, spec: T) -> Self {
        Self {
            id,
            children: HashSet::default(),
            spec,
            state: None,
        }
    }

    pub const fn id(&self) -> &Identity {
        &self.id
    }

    pub const fn children(&self) -> &HashSet<Identity> {
        &self.children
    }

    pub const fn children_mut(&mut self) -> &mut HashSet<Identity> {
        &mut self.children
    }

    pub const fn spec(&self) -> &T {
        &self.spec
    }

    pub const fn spec_mut(&mut self) -> &mut T {
        &mut self.spec
    }

    pub const fn state(&self) -> Option<&T::State> {
        self.state.as_ref()
    }

    pub const fn state_mut(&mut self) -> Option<&mut T::State> {
        self.state.as_mut()
    }

    pub const fn state_opt(&self) -> &Option<T::State> {
        &self.state
    }

    pub const fn state_opt_mut(&mut self) -> &mut Option<T::State> {
        &mut self.state
    }
}

impl<T> UserConfigResource<T>
where
    T: Specification,
{
    pub const fn new(meta: ResourceMeta<T>) -> Self {
        Self { meta }
    }

    pub const fn meta(&self) -> &ResourceMeta<T> {
        &self.meta
    }

    pub const fn meta_mut(&mut self) -> &mut ResourceMeta<T> {
        &mut self.meta
    }
}

impl<T> DynamicResource<T>
where
    T: Specification,
{
    pub fn new(meta: ResourceMeta<T>, owner: Identity) -> Self {
        Self {
            meta,
            owner,
            dependencies: HashSet::default(),
        }
    }

    pub const fn meta(&self) -> &ResourceMeta<T> {
        &self.meta
    }

    pub const fn meta_mut(&mut self) -> &mut ResourceMeta<T> {
        &mut self.meta
    }

    pub const fn owner(&self) -> &Identity {
        &self.owner
    }

    pub const fn dependencies(&self) -> &HashSet<Identity> {
        &self.dependencies
    }

    pub const fn dependencies_mut(&mut self) -> &mut HashSet<Identity> {
        &mut self.dependencies
    }
}

impl<T> From<UserConfigResource<T>> for Resource<T>
where
    T: Specification,
{
    fn from(value: UserConfigResource<T>) -> Self {
        Self::UserConfig(value)
    }
}

impl<T> From<DynamicResource<T>> for Resource<T>
where
    T: Specification,
{
    fn from(value: DynamicResource<T>) -> Self {
        Self::Dynamic(value)
    }
}

impl<T> From<ResourceMeta<T>> for UserConfigResource<T>
where
    T: Specification,
{
    fn from(value: ResourceMeta<T>) -> Self {
        Self::new(value)
    }
}

impl<T> TryFrom<ResourceMeta<T>> for v1::ResourceMeta
where
    T: Specification,
{
    type Error = String;

    fn try_from(value: ResourceMeta<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Some(value.id.into()),
            children: value.children.into_iter().map(From::from).collect(),
            spec: value
                .spec
                .into_bytes()
                .map_err(|_| "invalid spec".to_string())?,
            state: value
                .state
                .map(|v| {
                    v.into_bytes().map_err(|_| "invalid state".to_string())
                })
                .transpose()?
                .unwrap_or_default(),
        })
    }
}

impl<T> TryFrom<Resource<T>> for v1::MetaResource
where
    T: Specification,
{
    type Error = String;

    fn try_from(value: Resource<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            resource_type: Some(value.try_into()?),
        })
    }
}

impl<T> TryFrom<Resource<T>> for v1::meta_resource::ResourceType
where
    T: Specification,
{
    type Error = String;

    fn try_from(value: Resource<T>) -> Result<Self, Self::Error> {
        Ok(match value {
            Resource::UserConfig(res) => Self::UserConfig(res.try_into()?),
            Resource::Dynamic(res) => Self::Dynamic(res.try_into()?),
        })
    }
}

impl<T> TryFrom<UserConfigResource<T>> for v1::UserConfigResource
where
    T: Specification,
{
    type Error = String;

    fn try_from(value: UserConfigResource<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            meta: Some(value.meta.try_into()?),
        })
    }
}

impl<T> TryFrom<DynamicResource<T>> for v1::DynamicResource
where
    T: Specification,
{
    type Error = String;

    fn try_from(value: DynamicResource<T>) -> Result<Self, Self::Error> {
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

impl<T> TryFrom<v1::ResourceMeta> for ResourceMeta<T>
where
    T: Specification,
{
    type Error = String;

    fn try_from(value: v1::ResourceMeta) -> Result<Self, Self::Error> {
        dbg!(&value.spec);
        let raw: rmpv::Value = rmp_serde::from_slice(&value.spec).unwrap();
        dbg!(raw);
        Ok(Self {
            id: value.id.try_into()?,
            children: value
                .children
                .into_iter()
                .map(Identity::try_from)
                .try_collect()?,
            spec: rmp_serde::from_slice(&value.spec)
                .map_err(|_| "invalid spec".to_string())?,
            state: if value.state.is_empty() {
                None
            } else {
                rmp_serde::from_slice::<T::State>(&value.state)
                    .map(Some)
                    .map_err(|_| "invalid state".to_string())?
            },
        })
    }
}

impl<T> TryFrom<Option<v1::ResourceMeta>> for ResourceMeta<T>
where
    T: Specification,
{
    type Error = String;

    fn try_from(value: Option<v1::ResourceMeta>) -> Result<Self, Self::Error> {
        value
            .ok_or_else(|| "ResourceMeta is required".to_string())?
            .try_into()
    }
}

impl<T> TryFrom<v1::MetaResource> for Resource<T>
where
    T: Specification,
{
    type Error = String;

    fn try_from(value: v1::MetaResource) -> Result<Self, Self::Error> {
        match value
            .resource_type
            .ok_or_else(|| "ResourceType is required".to_string())?
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

impl<T> TryFrom<Option<v1::MetaResource>> for Resource<T>
where
    T: Specification,
{
    type Error = String;

    fn try_from(value: Option<v1::MetaResource>) -> Result<Self, Self::Error> {
        value
            .ok_or_else(|| "MetaResource is required".to_string())?
            .try_into()
    }
}

impl<T> TryFrom<v1::UserConfigResource> for UserConfigResource<T>
where
    T: Specification,
{
    type Error = String;

    fn try_from(value: v1::UserConfigResource) -> Result<Self, Self::Error> {
        Ok(Self {
            meta: value.meta.try_into()?,
        })
    }
}

impl<T> TryFrom<Option<v1::UserConfigResource>> for UserConfigResource<T>
where
    T: Specification,
{
    type Error = String;

    fn try_from(
        value: Option<v1::UserConfigResource>,
    ) -> Result<Self, Self::Error> {
        value
            .ok_or_else(|| "UserConfigResource is required".to_string())?
            .try_into()
    }
}

impl<T> TryFrom<v1::DynamicResource> for DynamicResource<T>
where
    T: Specification,
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

impl<T> TryFrom<Option<v1::DynamicResource>> for DynamicResource<T>
where
    T: Specification,
{
    type Error = String;

    fn try_from(
        value: Option<v1::DynamicResource>,
    ) -> Result<Self, Self::Error> {
        value
            .ok_or_else(|| "DynamicResource is required".to_string())?
            .try_into()
    }
}
