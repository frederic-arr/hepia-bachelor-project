use cos_api_sysmgr::proto::v1;
use cos_api_shared::Identity;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CreateConfig {
    pub id: Identity,
    pub spec: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CreateResource {
    pub id: Identity,
    pub owner: Identity,
    pub spec: Vec<u8>,
}

impl TryFrom<v1::ResourceCreateDynamicRequest> for CreateResource {
    type Error = ();

    fn try_from(
        value: v1::ResourceCreateDynamicRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.try_into()?,
            owner: value.owner.try_into()?,
            spec: value.spec,
        })
    }
}
