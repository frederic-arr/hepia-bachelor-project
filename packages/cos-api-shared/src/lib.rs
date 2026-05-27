#![feature(iterator_try_collect)]
#![feature(associated_type_defaults)]
#![feature(decl_macro)]

mod model;
pub mod proto;

pub use model::*;

pub trait Reconcilable: Specification {
    type CurrentState;
    type Data;
    type Error;
    type Output = ();
    type Plan;

    fn refresh(
        resource: &Resource<Self>,
        data: &mut Self::Data,
    ) -> impl Future<Output = Result<Option<Self::CurrentState>, Self::Error>> + Send;

    fn plan(
        resource: &Resource<Self>,
        data: &Self::Data,
        state: Option<&Self::CurrentState>,
    ) -> Result<Self::Plan, Self::Error>;

    fn apply(
        resource: &Resource<Self>,
        data: &mut Self::Data,
        plan: Self::Plan,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;
}

pub trait ReconcilableDriver {
    type CurrentState;
    type Data;
    type Error;
    type Output;
    type Plan;

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
        plan: Self::Plan,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;
}

impl<T> ReconcilableDriver for Resource<T>
where
    T: Specification + Reconcilable,
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
        plan: Self::Plan,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        T::apply(self, data, plan)
    }
}
