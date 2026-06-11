pub mod proto;

pub trait Reconcilable {
    const SCHEMA: &'static str;

    type Input;
    type State;
    type Plan;
    type Apply;
    type Output;
    type Context;

    fn refresh(
        ctx: &mut Self::Context,
        input: &Self::Input,
    ) -> impl Future<Output = Self::State>;

    fn plan(
        ctx: &mut Self::Context,
        input: &Self::Input,
        refreshed_state: &Self::State,
    ) -> impl Future<Output = Self::Plan>;

    fn apply(
        ctx: &mut Self::Context,
        input: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
    ) -> impl Future<Output = Self::Apply>;

    fn update(
        ctx: &mut Self::Context,
        input: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
        apply: &Self::Apply,
    ) -> impl Future<Output = Self::Output>;

    fn reconcile(
        ctx: &mut Self::Context,
        input: &Self::Input,
    ) -> impl Future<Output = Self::Output> {
        async {
            let refreshed_state = Self::refresh(ctx, input).await;
            let plan = Self::plan(ctx, input, &refreshed_state).await;
            let apply = Self::apply(ctx, input, &refreshed_state, &plan).await;

            Self::update(ctx, input, &refreshed_state, &plan, &apply).await
        }
    }
}
