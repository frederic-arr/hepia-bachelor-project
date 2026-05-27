use std::collections::HashSet;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::macros::*;
use crate::proto::v1;
use crate::{
    Identity,
    Specification,
    State,
    delegate_to_meta,
    impl_try_from_opt,
    impl_try_from_opt_bounds,
};

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
    status: ResourceStatus,
    state: Option<T::State>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceStatus {
    Running,
    Deleting,
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
    delegate_to_meta!(@rw-, T);

    pub const fn meta(&self) -> &ResourceMeta<T> {
        match self {
            Self::UserConfig(res) => &res.meta,
            Self::Dynamic(res) => &res.meta,
        }
    }

    pub const fn meta_mut(&mut self) -> &mut ResourceMeta<T> {
        match self {
            Self::UserConfig(res) => &mut res.meta,
            Self::Dynamic(res) => &mut res.meta,
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

    pub const fn maybe_dynamic_mut(
        &mut self,
    ) -> Option<&mut DynamicResource<T>> {
        match self {
            Self::Dynamic(res) => Some(res),
            Self::UserConfig(_) => None,
        }
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
            status: ResourceStatus::Running,
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

    pub const fn status(&self) -> &ResourceStatus {
        &self.status
    }

    pub const fn status_mut(&mut self) -> &mut ResourceStatus {
        &mut self.status
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
    delegate_to_meta!(@rw, T);

    pub const fn new(meta: ResourceMeta<T>) -> Self {
        Self { meta }
    }
}

impl<T> DynamicResource<T>
where
    T: Specification,
{
    delegate_to_meta!(@rw, T);

    pub fn try_new(
        meta: ResourceMeta<T>,
        owner: Identity,
    ) -> Result<Self, String> {
        if meta.id == owner {
            return Err(format!("self-referenced owner {owner}"));
        }

        Ok(Self {
            meta,
            owner,
            dependencies: HashSet::default(),
        })
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
        let mut meta = Self {
            id: Some(value.id.into()),
            children: value.children.into_iter().map(From::from).collect(),
            spec: value
                .spec
                .into_bytes()
                .map_err(|_| "invalid spec".to_string())?,
            status: v1::ResourceStatus::Unspecified.into(),
            state: value
                .state
                .map(|v| {
                    v.into_bytes().map_err(|_| "invalid state".to_string())
                })
                .transpose()?
                .unwrap_or_default(),
        };

        meta.set_status(value.status.try_into()?);
        Ok(meta)
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
        match value {
            Resource::UserConfig(res) => Ok(Self::UserConfig(res.try_into()?)),
            Resource::Dynamic(res) => Ok(Self::Dynamic(res.try_into()?)),
        }
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

impl TryFrom<ResourceStatus> for v1::ResourceStatus {
    type Error = String;

    fn try_from(value: ResourceStatus) -> Result<Self, Self::Error> {
        match value {
            ResourceStatus::Running => Ok(Self::Running),
            ResourceStatus::Deleting => Ok(Self::Deleting),
        }
    }
}

impl<T> TryFrom<v1::ResourceMeta> for ResourceMeta<T>
where
    T: Specification,
{
    type Error = String;

    fn try_from(value: v1::ResourceMeta) -> Result<Self, Self::Error> {
        let raw: rmpv::Value = rmp_serde::from_slice(&value.spec).unwrap();
        let status = value.status().try_into()?;
        Ok(Self {
            id: value.id.try_into()?,
            children: value
                .children
                .into_iter()
                .map(Identity::try_from)
                .try_collect()?,
            status,
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

impl TryFrom<v1::ResourceStatus> for ResourceStatus {
    type Error = String;

    fn try_from(value: v1::ResourceStatus) -> Result<Self, Self::Error> {
        match value {
            v1::ResourceStatus::Unspecified => {
                Err("unspecified resource status".to_string())
            }
            v1::ResourceStatus::Running => Ok(Self::Running),
            v1::ResourceStatus::Deleting => Ok(Self::Deleting),
        }
    }
}

impl_try_from_opt_bounds!(v1::ResourceMeta => ResourceMeta);
impl_try_from_opt_bounds!(v1::MetaResource => Resource);
impl_try_from_opt_bounds!(v1::UserConfigResource => UserConfigResource);
impl_try_from_opt_bounds!(v1::DynamicResource => DynamicResource);
impl_try_from_opt!(v1::ResourceStatus => ResourceStatus);
