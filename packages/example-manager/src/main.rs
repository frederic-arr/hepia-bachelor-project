#![feature(map_try_insert)]

use std::collections::{HashMap, HashSet};

use serde_json::Value;

/// Identity represents a unique object across the entire system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Identity {
    /// All objects of the same type share a common schema
    schema: String,

    /// Within a schema, each object can be differenciated using the name
    name: String,
}

#[derive(Debug)]
struct Resource {
    /// The unique identity of the resource
    id: Identity,

    /// List of identities of resources directly created by the current
    /// resource
    children: HashSet<Identity>,

    /// List of resources for which deletion will be blocked until the current
    /// resource is deleted or the dependency is removed
    depends_on: HashSet<Identity>,

    /// Desired state
    spec: Value,

    /// Actual state
    status: Value,
}

#[derive(Debug)]
enum Phase {
    Pending,
    Running,
    Terminating,
}

#[derive(Debug)]
struct SystemManager {
    resources: HashMap<Identity, Resource>,
}

impl SystemManager {
    fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    fn create_resource(&mut self, resource: Resource) -> Result<(), ()> {
        self.resources
            .try_insert(resource.id.clone(), resource)
            .map(|_| ())
            .map_err(|_| ())
    }

    fn delete_resource(
        &mut self,
        id: Identity,
    ) -> Result<(), HashSet<Identity>> {
        if let Some(children) = self.teardown_resource(&id) {
            return Err(children);
        }

        self.resources.remove(&id);
        Ok(())
    }

    fn teardown_resource(
        &mut self,
        id: &Identity,
    ) -> Option<HashSet<Identity>> {
        self.resources
            .entry(id.clone())
            .and_modify(|r| r.phase = Phase::Teardown);

        let children = self
            .resources
            .iter_mut()
            .filter_map(|(_, r)| r.owner.eq(id).then_some(r.id.clone()))
            .collect::<HashSet<_>>();

        if children.is_empty() {
            return None;
        }

        let mut tree = children.clone();
        for child in children {
            if let Some(grandchildren) = self.teardown_resource(&child) {
                tree.extend(grandchildren);
            }
        }

        Some(tree)
    }
}

fn main() {
    println!("Hello, world!");
}

// mod api {
//     pub struct ApiClient;

//     impl ApiClient {

//     }
// }

// mod netmgr {
//     struct NetworkManager;

//     impl NetworkManager {
//         pub async fn run(&self) {

//         }
//     }
// }
