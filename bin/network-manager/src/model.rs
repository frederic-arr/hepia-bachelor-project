// use cos_api_reconciler::proto::v1;
// use cos_api_shared::{Identity, Resource};
// use serde::de::DeserializeOwned;

// #[derive(Debug, Clone, Default, PartialEq, Eq)]
// pub struct ReconcileResource<Spec> {
//     pub id: Identity,
//     pub spec: Spec,
// }

// impl<Spec, Status> TryFrom<v1::ReconcileResourceRequest>
//     for ReconcileResource<Spec, Status>
// where
//     Spec: DeserializeOwned,
//     Status: DeserializeOwned,
// {
//     type Error = ();

//     fn try_from(
//         value: v1::ReconcileResourceRequest,
//     ) -> Result<Self, Self::Error> {
//         value.resource.try_into()
//     }
// }
