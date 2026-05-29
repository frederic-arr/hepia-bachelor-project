pub mod v1 {
    #![allow(
        clippy::all,
        clippy::pedantic,
        clippy::nursery,
        clippy::restriction,
        warnings,
        unknown_lints
    )]

    tonic::include_proto!("containeros.shared.v1");

    impl MetaResource {
        pub fn schema(&self) -> Option<&str> {
            let meta = match self.resource_type.as_ref()? {
                meta_resource::ResourceType::UserConfig(res) => {
                    res.meta.as_ref()?
                }
                meta_resource::ResourceType::Dynamic(res) => {
                    res.meta.as_ref()?
                }
            };

            let id = meta.id.as_ref()?;
            Some(&id.schema)
        }
    }
}
