use cos_api_shared::{
    DynamicResource,
    Identity,
    Resource,
    ResourceMeta,
    UserConfigResource,
};
use invariant_macros::invariant;
use tokio::time::Instant;

use super::*;

impl StateManager {
    pub fn config_create(&mut self, req: CreateConfig) -> Result<(), String> {
        let id = req.id.clone();
        let meta = ResourceMeta::<Payload>::new(req.id, req.spec.into());
        let resource = UserConfigResource::new(meta);

        self.resources
            .try_insert(id.clone(), resource.try_into().unwrap())
            .map(|_| ())
            .map_err(|_| "cannot create a duplicate resource".to_string())?;

        self.reconciliation_queue.insert_at(id, Instant::now());
        Ok(())
    }

    pub fn resource_create(
        &mut self,
        req: CreateResource,
    ) -> Result<(), String> {
        let id = req.id.clone();
        let meta = ResourceMeta::<Payload>::new(req.id, req.spec.into());
        let resource = DynamicResource::new(meta, req.owner);

        if self.resources.contains_key(resource.meta().id()) {
            return Err("cannot create a duplicate resource".to_string());
        }

        let inserted_in_owner = self
            .resources
            .get_mut(resource.owner())
            .ok_or_else(|| {
                "the resource owner should be a valid reference".to_string()
            })?
            .meta_mut()
            .children_mut()
            .insert(resource.meta().id().clone());

        invariant!(
            inserted_in_owner,
            "owner {} contains child {id} that does not exist in the resource \
             store",
            resource.owner(),
        );

        let exists_in_store = self
            .resources
            .insert(
                resource.meta().id().clone(),
                resource.try_into().unwrap(),
            )
            .is_some();

        invariant!(
            !exists_in_store,
            "resource {id} was absent during contains_key check but present \
             on insert"
        );

        self.reconciliation_queue.insert_at(id, Instant::now());
        Ok(())
    }

    pub fn resource_read(&self, id: &Identity) -> Option<&Resource<Payload>> {
        self.resources.get(id)
    }

    pub fn resource_read_user_config(
        &self,
        id: &Identity,
    ) -> Option<&UserConfigResource<Payload>> {
        self.resource_read(id)?.maybe_user_config()
    }

    pub fn resource_read_dynamic(
        &self,
        id: &Identity,
    ) -> Option<&DynamicResource<Payload>> {
        self.resource_read(id)?.maybe_dynamic()
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    mod config {
        use super::*;

        mod create {
            use super::*;

            fn setup() -> (StateManager, CreateConfig) {
                (
                    StateManager::new(),
                    CreateConfig {
                        id: Identity::default(),
                        spec: vec![].into(),
                    },
                )
            }

            #[test]
            fn basic_succeeds() {
                let (mut svc, res) = setup();

                svc.config_create(res.clone()).unwrap();

                assert_eq!(svc.resources.len(), 1);
                assert_matches!(
                    svc.resource_read_user_config(&res.id),
                    Some(_)
                );
            }

            #[test]
            fn fails_if_already_exists() {
                let (mut svc, res) = setup();
                svc.config_create(res.clone()).unwrap();

                svc.config_create(res.clone()).unwrap_err();

                assert_eq!(svc.resources.len(), 1);
                assert_matches!(
                    svc.resource_read_user_config(&res.id),
                    Some(_)
                );
            }
        }
    }

    mod dynamic {
        use super::*;

        mod create {
            use super::*;

            fn setup() -> (StateManager, CreateResource) {
                let mut svc = StateManager::new();
                let cfg = CreateConfig {
                    id: Identity::new(
                        "my-schema".to_string(),
                        "my-id".to_string(),
                    ),
                    spec: vec![].into(),
                };
                svc.config_create(cfg.clone()).unwrap();
                (
                    svc,
                    CreateResource {
                        id: Identity::default(),
                        owner: cfg.id,
                        spec: vec![],
                    },
                )
            }

            #[test]
            fn basic_succeeds() {
                let (mut svc, res) = setup();

                svc.resource_create(res.clone()).unwrap();

                assert_eq!(svc.resources.len(), 2);
                assert_matches!(svc.resource_read_dynamic(&res.id), Some(_));
            }

            #[test]
            fn fails_if_already_exists() {
                let (mut svc, res) = setup();
                svc.resource_create(res.clone()).unwrap();

                svc.resource_create(res.clone()).unwrap_err();

                assert_eq!(svc.resources.len(), 2);
                assert_matches!(svc.resource_read_dynamic(&res.id), Some(_));
            }

            #[test]
            fn fails_if_owner_is_invalid() {
                let mut svc = StateManager::new();
                let res = CreateResource::default();

                svc.resource_create(res.clone()).unwrap_err();

                assert_eq!(svc.resources.len(), 0);
                assert_eq!(svc.resource_read(&res.id), None);
            }
        }
    }
}
