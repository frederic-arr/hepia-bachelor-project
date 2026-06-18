pub mod proto;

pub trait Reconcilable {
    const SCHEMA: &'static str;

    type Input;
    type State;
    type Plan;
    type Apply;
    type Output;
    type Context;
    type Error;

    fn refresh(
        ctx: &mut Self::Context,
        input: &Self::Input,
    ) -> impl Future<Output = Result<Self::State, Self::Error>>;

    fn plan(
        ctx: &mut Self::Context,
        input: &Self::Input,
        refreshed_state: &Self::State,
    ) -> impl Future<Output = Result<Self::Plan, Self::Error>>;

    fn apply(
        ctx: &mut Self::Context,
        input: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
    ) -> impl Future<Output = Result<Self::Apply, Self::Error>>;

    fn update(
        ctx: &mut Self::Context,
        input: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
        apply: &Self::Apply,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>>;

    fn reconcile(
        ctx: &mut Self::Context,
        input: &Self::Input,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> {
        async {
            let refreshed_state = Self::refresh(ctx, input).await?;
            let plan = Self::plan(ctx, input, &refreshed_state).await?;
            let apply =
                Self::apply(ctx, input, &refreshed_state, &plan).await?;

            Self::update(ctx, input, &refreshed_state, &plan, &apply).await
        }
    }
}
