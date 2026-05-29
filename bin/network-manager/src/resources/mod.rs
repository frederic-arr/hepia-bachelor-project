mod link_config;
mod link;

pub use link_config::*;
pub use link::*;

pub trait Reconcilable {
    const SCHEMA: &'static str;

    type Input;
    type State;
    type Plan;
    type Apply;
    type Output;

    fn refresh(input: &Self::Input) -> impl Future<Output = Self::State>;

    fn plan(
        input: &Self::Input,
        refreshed_state: &Self::State,
    ) -> impl Future<Output = Self::Plan>;

    fn apply(
        input: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
    ) -> impl Future<Output = Self::Apply>;

    fn update(
        input: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
        apply: &Self::Apply,
    ) -> impl Future<Output = Self::Output>;

    fn reconcile(input: &Self::Input) -> impl Future<Output = Self::Output> {
        async {
            let refreshed_state = Self::refresh(input).await;
            let plan = Self::plan(input, &refreshed_state).await;
            let apply = Self::apply(input, &refreshed_state, &plan).await;

            Self::update(input, &refreshed_state, &plan, &apply).await
        }
    }
}
