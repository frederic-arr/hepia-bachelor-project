use std::collections::HashSet;
use std::time::SystemTime;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    spec: ResourceSpec<T>,
    state: ResourceState<T>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize",
    deserialize = "T: DeserializeOwned"
))]
pub enum ResourceSpec<T>
where
    T: Specification,
{
    Running(T),
    Draining(T),
    Deleting(T),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize",
    deserialize = "T: DeserializeOwned"
))]
pub enum ResourceState<T>
where
    T: Specification,
{
    Unset,
    Pending {
        state: T::State,
        state_at: SystemTime,
    },
    Error2 {
        error: String,
        state: T::State,
        state_at: SystemTime,
    },
    Ready {
        state: T::State,
        state_at: SystemTime,
    },
    Completed {
        state: T::State,
        state_at: SystemTime,
    },
    RefreshError {
        error: String,
        state: T::State,
        state_at: SystemTime,
    },
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
    pub fn new(id: Identity, spec: ResourceSpec<T>) -> Self {
        Self {
            id,
            children: HashSet::default(),
            spec,
            state: ResourceState::Unset,
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

    pub const fn spec(&self) -> &ResourceSpec<T> {
        &self.spec
    }

    pub const fn spec_mut(&mut self) -> &mut ResourceSpec<T> {
        &mut self.spec
    }

    pub fn spec_inner(&self) -> &T {
        self.spec.inner()
    }

    pub fn spec_inner_mut(&mut self) -> &mut T {
        self.spec.inner_mut()
    }

    pub const fn state(&self) -> &ResourceState<T> {
        &self.state
    }

    pub const fn state_mut(&mut self) -> &mut ResourceState<T> {
        &mut self.state
    }

    pub fn state_inner(&self) -> Option<&T::State> {
        self.state.inner()
    }

    pub fn state_inner_mut(&mut self) -> Option<&mut T::State> {
        self.state.inner_mut()
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

impl<T> ResourceSpec<T>
where
    T: Specification,
{
    fn into_inner(self) -> T {
        match self {
            ResourceSpec::Running(spec) => spec,
            ResourceSpec::Draining(spec) => spec,
            ResourceSpec::Deleting(spec) => spec,
        }
    }

    fn inner(&self) -> &T {
        match self {
            ResourceSpec::Running(spec) => spec,
            ResourceSpec::Draining(spec) => spec,
            ResourceSpec::Deleting(spec) => spec,
        }
    }

    fn inner_mut(&mut self) -> &mut T {
        match self {
            ResourceSpec::Running(spec) => spec,
            ResourceSpec::Draining(spec) => spec,
            ResourceSpec::Deleting(spec) => spec,
        }
    }
}

impl<T> ResourceState<T>
where
    T: Specification,
{
    fn into_inner(self) -> Option<T::State> {
        match self {
            ResourceState::Unset => None,
            ResourceState::Pending { state, .. } => Some(state),
            ResourceState::Error2 { state, .. } => Some(state),
            ResourceState::Ready { state, .. } => Some(state),
            ResourceState::Completed { state, .. } => Some(state),
            ResourceState::RefreshError { state, .. } => Some(state),
        }
    }

    fn inner(&self) -> Option<&T::State> {
        match self {
            ResourceState::Unset => None,
            ResourceState::Pending { state, .. } => Some(state),
            ResourceState::Error2 { state, .. } => Some(state),
            ResourceState::Ready { state, .. } => Some(state),
            ResourceState::Completed { state, .. } => Some(state),
            ResourceState::RefreshError { state, .. } => Some(state),
        }
    }

    fn inner_mut(&mut self) -> Option<&mut T::State> {
        match self {
            ResourceState::Unset => None,
            ResourceState::Pending { state, .. } => Some(state),
            ResourceState::Error2 { state, .. } => Some(state),
            ResourceState::Ready { state, .. } => Some(state),
            ResourceState::Completed { state, .. } => Some(state),
            ResourceState::RefreshError { state, .. } => Some(state),
        }
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
            spec: Some(value.spec.try_into()?),
            state: Some(value.state.try_into()?),
        })
    }
}

impl<T> TryFrom<ResourceSpec<T>> for v1::resource_meta::Spec
where
    T: Specification,
{
    type Error = String;

    fn try_from(value: ResourceSpec<T>) -> Result<Self, Self::Error> {
        use v1::resource_meta::Spec::*;

        match value {
            ResourceSpec::Running(spec) => spec
                .into_bytes()
                .map(SpecRunning)
                .map_err(|_| "invalid spec".to_string()),
            ResourceSpec::Draining(spec) => spec
                .into_bytes()
                .map(SpecDraining)
                .map_err(|_| "invalid spec".to_string()),
            ResourceSpec::Deleting(spec) => spec
                .into_bytes()
                .map(SpecDeleting)
                .map_err(|_| "invalid spec".to_string()),
        }
    }
}

impl<T> TryFrom<ResourceState<T>> for v1::resource_meta::State
where
    T: Specification,
{
    type Error = String;

    fn try_from(value: ResourceState<T>) -> Result<Self, Self::Error> {
        use v1::resource_meta::State::*;

        match value {
            ResourceState::Unset => Ok(StateUnset(())),
            ResourceState::Pending { state, state_at } => state
                .into_bytes()
                .map_err(|_| "invalid state".to_string())
                .map(|state| {
                    StatePending(v1::ResourceState {
                        state,
                        state_at: Some(state_at.into()),
                    })
                }),
            ResourceState::Ready { state, state_at } => state
                .into_bytes()
                .map_err(|_| "invalid state".to_string())
                .map(|state| {
                    StateReady(v1::ResourceState {
                        state,
                        state_at: Some(state_at.into()),
                    })
                }),
            ResourceState::Completed { state, state_at } => state
                .into_bytes()
                .map_err(|_| "invalid state".to_string())
                .map(|state| {
                    StateCompleted(v1::ResourceState {
                        state,
                        state_at: Some(state_at.into()),
                    })
                }),
            ResourceState::Error2 {
                state,
                state_at,
                error,
            } => state
                .into_bytes()
                .map_err(|_| "invalid state".to_string())
                .map(|state| {
                    StateError(v1::ResourceStateError {
                        state,
                        state_at: Some(state_at.into()),
                        error,
                    })
                }),
            ResourceState::RefreshError {
                state,
                state_at,
                error,
            } => state
                .into_bytes()
                .map_err(|_| "invalid state".to_string())
                .map(|state| {
                    StateError(v1::ResourceStateError {
                        state,
                        state_at: Some(state_at.into()),
                        error,
                    })
                }),
        }
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

impl<T> TryFrom<v1::ResourceMeta> for ResourceMeta<T>
where
    T: Specification,
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
            spec: value.spec.try_into()?,
            state: value.state.try_into()?,
        })
    }
}

impl<T> TryFrom<v1::resource_meta::Spec> for ResourceSpec<T>
where
    T: Specification,
{
    type Error = String;

    fn try_from(value: v1::resource_meta::Spec) -> Result<Self, Self::Error> {
        use v1::resource_meta::Spec::*;

        match value {
            SpecRunning(data) => rmp_serde::from_slice::<T>(&data)
                .map(Self::Running)
                .map_err(|_| "invalid spec".to_string()),
            SpecDraining(data) => rmp_serde::from_slice::<T>(&data)
                .map(Self::Draining)
                .map_err(|_| "invalid spec".to_string()),
            SpecDeleting(data) => rmp_serde::from_slice::<T>(&data)
                .map(Self::Deleting)
                .map_err(|_| "invalid spec".to_string()),
        }
    }
}

impl<T> TryFrom<v1::resource_meta::State> for ResourceState<T>
where
    T: Specification,
{
    type Error = String;

    fn try_from(
        value: v1::resource_meta::State,
    ) -> Result<Self, <Self as TryFrom<v1::resource_meta::State>>::Error> {
        use v1::resource_meta::State::*;

        match value {
            StateUnset(_) => Ok(Self::Unset),
            StatePending(v1::ResourceState { state, state_at }) => {
                rmp_serde::from_slice::<T::State>(&state)
                    .map_err(|_| "invalid state".to_string())
                    .map(|state| Self::Pending {
                        state,
                        state_at: state_at.unwrap().try_into().unwrap(),
                    })
            }
            StateReady(v1::ResourceState { state, state_at }) => {
                rmp_serde::from_slice::<T::State>(&state)
                    .map_err(|_| "invalid state".to_string())
                    .map(|state| Self::Ready {
                        state,
                        state_at: state_at.unwrap().try_into().unwrap(),
                    })
            }
            StateCompleted(v1::ResourceState { state, state_at }) => {
                rmp_serde::from_slice::<T::State>(&state)
                    .map_err(|_| "invalid state".to_string())
                    .map(|state| Self::Completed {
                        state,
                        state_at: state_at.unwrap().try_into().unwrap(),
                    })
            }
            StateError(v1::ResourceStateError {
                state,
                state_at,
                error,
            }) => rmp_serde::from_slice::<T::State>(&state)
                .map_err(|_| "invalid state".to_string())
                .map(|state| Self::Error2 {
                    state,
                    state_at: state_at.unwrap().try_into().unwrap(),
                    error,
                }),
            StateRefreshError(v1::ResourceStateError {
                state,
                state_at,
                error,
            }) => rmp_serde::from_slice::<T::State>(&state)
                .map_err(|_| "invalid state".to_string())
                .map(|state| Self::RefreshError {
                    state,
                    state_at: state_at.unwrap().try_into().unwrap(),
                    error,
                }),
        }
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

impl<T> TryFrom<v1::MetaResource> for UserConfigResource<T>
where
    T: Specification,
{
    type Error = String;

    fn try_from(value: v1::MetaResource) -> Result<Self, Self::Error> {
        match value
            .resource_type
            .ok_or_else(|| "ResourceType is required".to_string())?
        {
            v1::meta_resource::ResourceType::UserConfig(res) => res.try_into(),
            v1::meta_resource::ResourceType::Dynamic(res) => {
                Err("cannot convert dynamic resource to user config"
                    .to_string())
            }
        }
    }
}

impl<T> TryFrom<v1::MetaResource> for DynamicResource<T>
where
    T: Specification,
{
    type Error = String;

    fn try_from(value: v1::MetaResource) -> Result<Self, Self::Error> {
        match value
            .resource_type
            .ok_or_else(|| "ResourceType is required".to_string())?
        {
            v1::meta_resource::ResourceType::Dynamic(res) => res.try_into(),
            v1::meta_resource::ResourceType::UserConfig(res) => {
                Err("cannot convert user config to dynamic resource"
                    .to_string())
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

// impl TryFrom<v1::ResourceStatus> for ResourceStatus {
//     type Error = String;

//     fn try_from(value: v1::ResourceStatus) -> Result<Self, Self::Error> {
//         match value {
//             v1::ResourceStatus::Unspecified => {
//                 Err("unspecified resource status".to_string())
//             }
//             v1::ResourceStatus::Running => Ok(Self::Running),
//             v1::ResourceStatus::Deleting => Ok(Self::Deleting),
//         }
//     }
// }

impl_try_from_opt_bounds!(v1::ResourceMeta => ResourceMeta);
impl_try_from_opt_bounds!(v1::resource_meta::Spec => ResourceSpec);
impl_try_from_opt_bounds!(v1::resource_meta::State => ResourceState);
impl_try_from_opt_bounds!(v1::MetaResource => Resource);
impl_try_from_opt_bounds!(v1::UserConfigResource => UserConfigResource);
impl_try_from_opt_bounds!(v1::DynamicResource => DynamicResource);
// impl_try_from_opt!(v1::ResourceStatus => ResourceStatus);
