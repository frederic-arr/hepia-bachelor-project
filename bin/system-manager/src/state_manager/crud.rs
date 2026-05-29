// use std::collections::HashSet;
// use std::collections::hash_map::Entry;
// use std::ops::Not;
// use std::time::Duration;

// use cos_api_shared::{
//     DynamicResource,
//     Identity,
//     Resource,
//     ResourceMeta,
//     ResourceSpec,
//     UserConfigResource,
// };
// use invariant_macros::{invariant, invariant_violation};
// use tokio::time::Instant;

// use super::{
//     CreateDynamicResource,
//     CreateUserConfigResource,
//     Payload,
//     StateManager,
// };

// impl StateManager {
//     pub fn resource_user_config_create(
//         &mut self,
//         req: CreateUserConfigResource,
//     ) -> Result<(), String> {
//         let id = req.id.clone();
//         let meta = ResourceMeta::<Payload>::new(
//             req.id,
//             ResourceSpec::Running(req.spec),
//         );
//         let resource = UserConfigResource::new(meta);

//         self.resources
//             .try_insert(id.clone(), resource.into())
//             .map(|_| ())
//             .map_err(|_| "cannot create a duplicate resource".to_string())?;

//         self.schedule_reconcile_at_latest(
//             id,
//             Instant::now() + Duration::from_secs(5),
//         );
//         Ok(())
//     }

//     pub fn resource_dynamic_validate_bulk(
//         &mut self,
//         reqs: Vec<CreateDynamicResource>,
//     ) -> Result<Vec<DynamicResource<Payload>>, String> {
//         let size = reqs.len();
//         reqs.into_iter()
//             .try_fold(
//                 (
//                     HashSet::with_capacity(size),
//                     Vec::with_capacity(size),
//                 ),
//                 |(mut seen, mut acc), req| {
//                     if !seen.insert(req.id.clone()) {
//                         return Err(format!(
//                             "duplicate id in batch: {}",
//                             req.id
//                         ));
//                     }

//                     let meta = ResourceMeta::<Payload>::new(
//                         req.id.clone(),
//                         ResourceSpec::Running(req.spec.into()),
//                     );
//                     let resource = DynamicResource::try_new(meta,
// req.owner)?;

//                     match (
//                         self.resources.contains_key(resource.owner()),
//                         self.resources.contains_key(resource.id()),
//                     ) {
//                         (true, false) => acc.push(resource),
//                         (_, true) => {
//                             return Err(format!(
//                                 "resource {} already exists",
//                                 resource.id()
//                             ));
//                         }
//                         (false, false) => {
//                             return Err(format!(
//                                 "owner {} does not exist",
//                                 resource.owner()
//                             ));
//                         }
//                     }

//                     Ok((seen, acc))
//                 },
//             )
//             .map(|(_, acc)| acc)

//         // TODO: Validate with reconciler
//     }

//     pub fn resource_dynamic_create_bulk(
//         &mut self,
//         reqs: Vec<CreateDynamicResource>,
//         schedule: Option<Instant>,
//     ) -> Result<(), String> {
//         // Either everything is inserted or nothing is
//         let validated = self.resource_dynamic_validate_bulk(reqs)?;

//         // INVARIANT: At this point, all insert should succeed because they
// have         // been validated
//         for resource in validated {
//             let id = resource.id().clone();
//             let Some(owner) = self.resources.get_mut(resource.owner()) else {
//                 invariant_violation!(
//                     "existence of owner {} should have been checked during \
//                      validation",
//                     resource.owner(),
//                 );
//             };

//             let inserted_in_owner =
//                 owner.children_mut().insert(resource.id().clone());

//             invariant!(
//                 inserted_in_owner,
//                 "owner {} contains child {id} before it has been created",
//                 owner.id(),
//             );

//             let exists_in_store = self
//                 .resources
//                 .insert(resource.id().clone(), resource.into())
//                 .is_some();

//             invariant!(
//                 !exists_in_store,
//                 "resource {id} was absent at time of check, but present when
// \                  inserted"
//             );

//             if let Some(when) = schedule {
//                 self.schedule_reconcile_at_latest(id, when);
//             }
//         }

//         Ok(())
//     }

//     pub fn resource_read(&self, id: &Identity) -> Option<&Resource<Payload>>
// {         self.resources.get(id)
//     }

//     pub fn resource_read_mut(
//         &mut self,
//         id: &Identity,
//     ) -> Option<&mut Resource<Payload>> {
//         self.resources.get_mut(id)
//     }

//     pub fn resource_read_user_config(
//         &self,
//         id: &Identity,
//     ) -> Option<&UserConfigResource<Payload>> {
//         self.resource_read(id)?.maybe_user_config()
//     }

//     pub fn resource_read_user_config_mut(
//         &mut self,
//         id: &Identity,
//     ) -> Option<&mut UserConfigResource<Payload>> {
//         self.resource_read_mut(id)?.maybe_user_config_mut()
//     }

//     pub fn resource_read_dynamic(
//         &self,
//         id: &Identity,
//     ) -> Option<&DynamicResource<Payload>> {
//         self.resource_read(id)?.maybe_dynamic()
//     }

//     pub fn resource_read_dynamic_mut(
//         &mut self,
//         id: &Identity,
//     ) -> Option<&mut DynamicResource<Payload>> {
//         self.resource_read_mut(id)?.maybe_dynamic_mut()
//     }
// }

// // #[cfg(test)]
// // mod tests {
// //     use std::assert_matches;

// //     use super::*;

// //     mod config {
// //         use super::*;

// //         mod create {
// //             use super::*;

// //             fn setup() -> (StateManager, CreateUserConfigResource) {
// //                 (
// //                     StateManager::new(),
// //                     CreateUserConfigResource {
// //                         id: Identity::default(),
// //                         spec: vec![].into(),
// //                     },
// //                 )
// //             }

// //             #[tokio::test]
// //             async fn basic_succeeds() {
// //                 let (mut svc, res) = setup();

// //                 svc.resource_user_config_create(res.clone()).unwrap();

// //                 assert_eq!(svc.resources.len(), 1);
// //                 assert_matches!(
// //                     svc.resource_read_user_config(&res.id),
// //                     Some(_)
// //                 );
// //                 assert_matches!(svc.get_scheduled_when(&res.id), Some(_));
// //             }

// //             #[tokio::test]
// //             async fn fails_if_already_exists() {
// //                 let (mut svc, res) = setup();
// //                 svc.resource_user_config_create(res.clone()).unwrap();
// //                 let existing_schedule = svc.get_scheduled_when(&res.id);

// //                 svc.resource_user_config_create(res.clone()).unwrap_err();

// //                 assert_eq!(svc.resources.len(), 1);
// //                 assert_matches!(
// //                     svc.resource_read_user_config(&res.id),
// //                     Some(_)
// //                 );
// //                 assert_eq!(svc.get_scheduled_when(&res.id),
// // existing_schedule);             }
// //         }
// //     }

// //     mod dynamic {
// //         use super::*;

// //         mod create {
// //             use super::*;

// //             fn setup() -> (StateManager, CreateDynamicResource) {
// //                 let mut svc = StateManager::new();
// //                 let cfg = CreateUserConfigResource {
// //                     id: Identity::new("_".to_string(),
// "root".to_string()), //                     spec: vec![].into(),
// //                 };
// //                 svc.resource_user_config_create(cfg.clone()).unwrap();
// //                 (
// //                     svc,
// //                     CreateDynamicResource {
// //                         id: Identity::default(),
// //                         owner: cfg.id,
// //                         spec: vec![],
// //                     },
// //                 )
// //             }

// //             mod happy_path {
// //                 use super::*;

// //                 #[tokio::test]
// //                 async fn succeeds_when_owner_exists_and_not_existing() {
// //                     let (mut svc, res) = setup();

// //                     svc.resource_dynamic_create(res.clone()).unwrap();

// //                     assert_eq!(svc.resources.len(), 2);
// //                     assert_matches!(
// //                         svc.resource_read_dynamic(&res.id),
// //                         Some(_)
// //                     );
// //                     assert_matches!(svc.get_scheduled_when(&res.id),
// // Some(_));                 }
// //             }

// //             mod error_path {
// //                 use super::*;

// //                 mod missing_owner {
// //                     use super::*;

// //                     #[tokio::test]
// //                     async fn fails_and_does_not_insert_when_not_existing()
// { //                         let mut svc = StateManager::new();
// //                         let res = CreateDynamicResource::default();

// // svc.resource_dynamic_create(res.clone()).unwrap_err();

// //                         assert_eq!(svc.resources.len(), 0);
// //                         assert_eq!(svc.resource_read(&res.id), None);
// //                         assert_matches!(svc.get_scheduled_when(&res.id),
// // None);                     }
// //                 }

// //                 mod duplicate {
// //                     use super::*;

// //                     #[tokio::test]
// //                     async fn fails_and_does_not_update_when_same_owner() {
// //                         let (mut svc, res) = setup();
// //                         svc.resource_dynamic_create(res.clone()).unwrap();
// //                         let existing =
// //                             svc.resource_read_dynamic(&res.id).cloned();
// //                         let existing_when =
// svc.get_scheduled_when(&res.id);

// // svc.resource_dynamic_create(res.clone()).unwrap_err();

// //                         assert_eq!(svc.resources.len(), 2);
// //                         assert_eq!(
// //                             svc.resource_read_dynamic(&res.id).cloned(),
// //                             existing
// //                         );
// //                         assert_eq!(
// //                             svc.get_scheduled_when(&res.id),
// //                             existing_when
// //                         );
// //                     }

// //                     #[tokio::test]
// //                     async fn
// fails_and_does_not_update_when_different_owner() // {
// let (mut svc, res) = setup(); //
// svc.resource_dynamic_create(res.clone()).unwrap(); //
// let existing = //
// svc.resource_read_dynamic(&res.id).cloned(); //                         let
// existing_when = svc.get_scheduled_when(&res.id);

// //                         let other_owner_id = Identity::new(
// //                             "_".to_string(),
// //                             "other-owner".to_string(),
// //                         );
// //                         svc.resource_user_config_create(
// //                             CreateUserConfigResource {
// //                                 id: other_owner_id.clone(),
// //                                 spec: vec![].into(),
// //                             },
// //                         )
// //                         .unwrap();

// //                         svc.resource_dynamic_create(CreateDynamicResource
// { //                             owner: other_owner_id.clone(),
// //                             ..res.clone()
// //                         })
// //                         .unwrap_err();

// //                         assert_eq!(svc.resources.len(), 3);
// //                         assert_eq!(
// //                             svc.resource_read_dynamic(&res.id).cloned(),
// //                             existing
// //                         );
// //                         assert_eq!(
// //                             svc.get_scheduled_when(&res.id),
// //                             existing_when
// //                         );
// //                     }
// //                 }

// //                 mod self_reference {
// //                     use super::*;

// //                     #[tokio::test]
// //                     async fn fails_and_does_not_insert_when_not_existing()
// { //                         let mut svc = StateManager::new();
// //                         let id = Identity::default();

// //                         svc.resource_dynamic_create(CreateDynamicResource
// { //                             id: id.clone(),
// //                             owner: id.clone(),
// //                             spec: vec![],
// //                         })
// //                         .unwrap_err();

// //                         assert_eq!(svc.resources.len(), 0);
// //                         assert_eq!(svc.resource_read(&id), None);
// //                         assert_matches!(svc.get_scheduled_when(&id),
// None); //                     }
// //                 }
// //             }
// //         }
// //     }
// // }
