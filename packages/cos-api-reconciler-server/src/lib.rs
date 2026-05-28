pub mod proto;

use cos_api_reconciler::proto::v1::{
    CreateDynamicResourceRequest,
    ReconcileDynamicResourceResponse,
};
use cos_api_shared::{
    DynamicResource,
    Resource,
    Specification,
    UserConfigResource,
};

pub trait Reconcilable: Specification {
    type Resource;
    type CurrentState: Send;
    type Data: Send;
    type Error: Into<tonic::Status>;
    type Output: Into<ReconcileDynamicResourceResponse>;
    type Plan: Send;

    fn refresh(
        resource: &Self::Resource,
        data: &mut Self::Data,
    ) -> impl Future<Output = Result<Option<Self::CurrentState>, Self::Error>> + Send;

    fn plan(
        resource: &Self::Resource,
        data: &Self::Data,
        state: Option<&Self::CurrentState>,
    ) -> Result<Self::Plan, Self::Error>;

    fn apply(
        resource: &Self::Resource,
        data: &mut Self::Data,
        state: Option<&Self::CurrentState>,
        plan: &Self::Plan,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;
}

pub trait ReconcilableDriver: Sync {
    type CurrentState: Send;
    type Data: Send;
    type Error: Into<tonic::Status>;
    type Output: Into<ReconcileDynamicResourceResponse>;
    type Plan: Send;

    fn refresh(
        &self,
        data: &mut Self::Data,
    ) -> impl Future<Output = Result<Option<Self::CurrentState>, Self::Error>> + Send;

    fn plan(
        &self,
        data: &Self::Data,
        state: Option<&Self::CurrentState>,
    ) -> Result<Self::Plan, Self::Error>;

    fn apply(
        &self,
        data: &mut Self::Data,
        state: Option<&Self::CurrentState>,
        plan: &Self::Plan,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;

    #[inline]
    fn reconcile(
        &self,
        data: &mut Self::Data,
    ) -> impl Future<
        Output = Result<ReconcileDynamicResourceResponse, tonic::Status>,
    > + Send {
        async {
            let state = self.refresh(data).await.map_err(Into::into)?;
            let plan = self.plan(data, state.as_ref()).map_err(Into::into)?;
            self.apply(data, state.as_ref(), &plan)
                .await
                .map(Into::into)
                .map_err(Into::into)
        }
    }
}

impl<T> ReconcilableDriver for Resource<T>
where
    T: Specification + Reconcilable<Resource = Resource<T>>,
{
    type CurrentState = T::CurrentState;
    type Data = T::Data;
    type Error = T::Error;
    type Output = T::Output;
    type Plan = T::Plan;

    #[inline]
    fn refresh(
        &self,
        data: &mut Self::Data,
    ) -> impl Future<Output = Result<Option<Self::CurrentState>, Self::Error>> + Send
    {
        T::refresh(self, data)
    }

    #[inline]
    fn plan(
        &self,
        data: &Self::Data,
        state: Option<&Self::CurrentState>,
    ) -> Result<Self::Plan, Self::Error> {
        T::plan(self, data, state)
    }

    #[inline]
    fn apply(
        &self,
        data: &mut Self::Data,
        state: Option<&Self::CurrentState>,
        plan: &Self::Plan,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        T::apply(self, data, state, plan)
    }
}

impl<T> ReconcilableDriver for UserConfigResource<T>
where
    T: Specification + Reconcilable<Resource = UserConfigResource<T>>,
{
    type CurrentState = T::CurrentState;
    type Data = T::Data;
    type Error = T::Error;
    type Output = T::Output;
    type Plan = T::Plan;

    #[inline]
    fn refresh(
        &self,
        data: &mut Self::Data,
    ) -> impl Future<Output = Result<Option<Self::CurrentState>, Self::Error>> + Send
    {
        T::refresh(self, data)
    }

    #[inline]
    fn plan(
        &self,
        data: &Self::Data,
        state: Option<&Self::CurrentState>,
    ) -> Result<Self::Plan, Self::Error> {
        T::plan(self, data, state)
    }

    #[inline]
    fn apply(
        &self,
        data: &mut Self::Data,
        state: Option<&Self::CurrentState>,
        plan: &Self::Plan,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        T::apply(self, data, state, plan)
    }
}

impl<T> ReconcilableDriver for DynamicResource<T>
where
    T: Specification + Reconcilable<Resource = DynamicResource<T>>,
{
    type CurrentState = T::CurrentState;
    type Data = T::Data;
    type Error = T::Error;
    type Output = T::Output;
    type Plan = T::Plan;

    #[inline]
    fn refresh(
        &self,
        data: &mut Self::Data,
    ) -> impl Future<Output = Result<Option<Self::CurrentState>, Self::Error>> + Send
    {
        T::refresh(self, data)
    }

    #[inline]
    fn plan(
        &self,
        data: &Self::Data,
        state: Option<&Self::CurrentState>,
    ) -> Result<Self::Plan, Self::Error> {
        T::plan(self, data, state)
    }

    #[inline]
    fn apply(
        &self,
        data: &mut Self::Data,
        state: Option<&Self::CurrentState>,
        plan: &Self::Plan,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        T::apply(self, data, state, plan)
    }
}
