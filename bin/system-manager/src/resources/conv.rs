use std::collections::HashSet;

use super::{ResourceMeta, UserConfigResource, UserConfigResourceCreate};
use crate::resources::{
    DynamicResource,
    DynamicResourceCreate,
    ResourceSpec,
    ResourceState,
};

impl From<UserConfigResourceCreate> for UserConfigResource {
    fn from(value: UserConfigResourceCreate) -> Self {
        Self {
            meta: ResourceMeta {
                id: value.id,
                children: HashSet::new(),
                spec: ResourceSpec::Running { spec: value.spec },
                state: ResourceState::Unset,
            },
        }
    }
}

impl From<DynamicResourceCreate> for DynamicResource {
    fn from(value: DynamicResourceCreate) -> Self {
        Self {
            meta: ResourceMeta {
                id: value.id,
                children: HashSet::new(),
                spec: ResourceSpec::Running { spec: value.spec },
                state: ResourceState::Unset,
            },
            owner: value.owner,
            dependencies: HashSet::new(),
        }
    }
}
