use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use bollard::Docker;
use bollard::query_parameters::{
    CreateImageOptionsBuilder,
    ListImagesOptionsBuilder,
    RemoveImageOptionsBuilder,
};
use cos_proto_reconciler::{
    Identity,
    Key,
    Phase,
    PrivateIdentity,
    Resource,
    ResourceResponse,
    Status,
    ValidateResponse,
};
use cos_proto_state::v1::ReconcileNowRequest;
use derive_builder::Builder;
use futures::StreamExt as _;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use crate::{RuntimeDerivedSpec, STATE_CLIENT};

static PULLING: LazyLock<Mutex<Option<JoinHandle<()>>>> =
    LazyLock::new(|| Mutex::new(None));

#[derive(Debug, Clone)]
pub struct ImageReconciler;

pub type ImageResource = Resource<ImageSpec, ImageDerivedSpec, ImageState>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImageSpec {}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImageDerivedSpec {
    pub runtime: String,
    pub image: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Deserialize, Serialize)]
#[builder(pattern = "mutable")]
pub struct ImageState {
    pub size: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImagePlan {
    Pull,
    Delete,
    Noop,
}

impl ImageReconciler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for ImageReconciler {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageReconciler {
    pub async fn validate(
        &self,
        key: Key,
        spec: ImageSpec,
        resource: Option<ImageResource>,
    ) -> Result<ValidateResponse<ImageDerivedSpec>> {
        if let Some(resource) = resource {
            self.validate_spec_change(&resource, &spec).await?;
        } else {
            self.validate_new_spec(&spec).await?;
        }

        let name = key.name.ok_or_else(|| anyhow!("missing name"))?;
        let Some((runtime, image)) = name.split_once('#') else {
            bail!("invalid image format");
        };

        let derived = ImageDerivedSpec {
            runtime: runtime.to_owned(),
            image: image.to_owned(),
        };

        Ok(ValidateResponse {
            derived_spec: derived.clone(),
            children: vec![],
            dependencies: Self::get_deps(&derived),
        })
    }

    // #[expect(clippy::too_many_lines, reason = "TODO")]
    pub async fn reconcile(
        &self,
        resource: ImageResource,
    ) -> Result<ResourceResponse<ImageState>> {
        let Some(rt) = resource
            .dependencies
            .iter()
            .find(|v| v.id.schema() == "container:runtime")
        else {
            return Ok(ResourceResponse {
                status: Status::NotReady,
                state: None,
                children: vec![],
                dependencies: Self::get_deps(&resource.derived_spec),
            });
        };

        let rt: RuntimeDerivedSpec =
            serde_json::from_value(rt.derived_spec.clone())?;

        let client =
            Docker::connect_with_host(&format!("tcp://127.0.0.1:{}", rt.port))?;

        let cx = match self.refresh(&resource, &client).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
                    state: resource.state,
                    children: vec![],
                    dependencies: Self::get_deps(&resource.derived_spec),
                });
            }
        };

        let state = &cx;
        let plan = match self.plan(&resource, cx.as_ref()).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
                    state: state.clone(),
                    children: vec![],
                    dependencies: Self::get_deps(&resource.derived_spec),
                });
            }
        };

        let () =
            match self.apply(&resource, &plan, state.as_ref(), &client).await {
                Ok(v) => v,
                Err(err) => {
                    return Ok(ResourceResponse {
                        status: Status::Error(format!("{err:#}").into()),
                        state: state.clone(),
                        children: vec![],
                        dependencies: Self::get_deps(&resource.derived_spec),
                    });
                }
            };

        let new_cx = match self.refresh(&resource, &client).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
                    state: state.clone(),
                    children: vec![],
                    dependencies: Self::get_deps(&resource.derived_spec),
                });
            }
        };

        let state = &new_cx;
        let new_plan = match self.plan(&resource, state.as_ref()).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
                    state: state.clone(),
                    children: vec![],
                    dependencies: Self::get_deps(&resource.derived_spec),
                });
            }
        };

        let status = match new_plan {
            ImagePlan::Noop if matches!(resource.phase, Phase::Teardown) => {
                Status::Deleted
            }
            ImagePlan::Noop => Status::Done,
            ImagePlan::Pull | ImagePlan::Delete => Status::NotReady,
        };

        Ok(ResourceResponse {
            status,
            state: state.clone(),
            children: vec![],
            dependencies: Self::get_deps(&resource.derived_spec),
        })
    }

    async fn refresh(
        &self,
        resource: &ImageResource,
        ctx: &Docker,
    ) -> Result<Option<ImageState>> {
        let mut filters = HashMap::new();
        filters.insert(
            "reference",
            vec![resource.derived_spec.image.as_str()],
        );

        let opts = ListImagesOptionsBuilder::default()
            .filters(&filters)
            .build();

        let images = ctx.list_images(Some(opts)).await?;
        let Some(image) = images.first() else {
            return Ok(None);
        };

        Ok(Some(ImageState { size: image.size }))
    }

    async fn plan(
        &self,
        resource: &ImageResource,
        cx: Option<&ImageState>,
    ) -> Result<ImagePlan> {
        let plan = match (&resource.phase, cx) {
            (Phase::Running, None) => ImagePlan::Pull,
            (Phase::Teardown, Some(_)) => ImagePlan::Delete,
            (Phase::Running | Phase::Shutdown, Some(_))
            | (Phase::Shutdown | Phase::Teardown, None) => ImagePlan::Noop,
        };

        Ok(plan)
    }

    async fn apply(
        &self,
        resource: &ImageResource,
        plan: &ImagePlan,
        _cx: Option<&ImageState>,
        ctx: &Docker,
    ) -> Result<()> {
        match plan {
            ImagePlan::Pull => {
                let mut pulling = PULLING.lock().await;
                if pulling.as_ref().is_none_or(JoinHandle::is_finished) {
                    let opts = CreateImageOptionsBuilder::default()
                        .from_image(&resource.derived_spec.image)
                        .build();

                    let mut stream = ctx.create_image(Some(opts), None, None);

                    let key = resource.id.key().clone();
                    let handle = tokio::spawn(async move {
                        while let Some(v) = stream.next().await {
                            if let Err(err) = v {
                                tracing::error!("image: {err}");
                                return;
                            }
                        }

                        let mut c = (*STATE_CLIENT).clone();
                        let raw = match serde_json::to_vec(&key) {
                            Ok(v) => v,
                            Err(_err) => return,
                        };

                        let _ = timeout(
                            Duration::from_secs(1),
                            c.reconcile_now(ReconcileNowRequest { raw }),
                        )
                        .await;
                    });

                    *pulling = Some(handle);
                }
                drop(pulling);

                Ok(())
            }
            ImagePlan::Delete => {
                let _deleted = ctx
                    .remove_image(
                        &resource.derived_spec.image,
                        Some(RemoveImageOptionsBuilder::new().build()),
                        None,
                    )
                    .await?;

                Ok(())
            }
            ImagePlan::Noop => Ok(()),
        }
    }

    async fn validate_new_spec(&self, _spec: &ImageSpec) -> Result<()> {
        Ok(())
    }

    async fn validate_spec_change(
        &self,
        _resource: &ImageResource,
        spec: &ImageSpec,
    ) -> Result<()> {
        self.validate_new_spec(spec).await
    }

    fn get_deps(spec: &ImageDerivedSpec) -> HashSet<Identity> {
        HashSet::from([Identity::Private(PrivateIdentity::Dynamic(Key {
            schema: "container:runtime".to_owned(),
            name: Some(spec.runtime.clone()),
        }))])
    }
}
