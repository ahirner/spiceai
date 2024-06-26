use async_trait::async_trait;
use snafu::prelude::*;

use crate::Runtime;
use spicepod::component::extension::Extension as ExtensionComponent;

#[allow(clippy::module_name_repetitions)]
pub type ExtensionManifest = ExtensionComponent;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unable to initialize extension: {source}"))]
    UnableToInitializeExtension {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("Unable to start extension: {source}"))]
    UnableToStartExtension {
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

///
/// Extension trait
///
/// This trait is used to define the interface for extensions to the Spice runtime.
#[async_trait]
#[allow(clippy::module_name_repetitions)]
pub trait Extension: Send + Sync {
    fn name(&self) -> &'static str;
    // fn metrics_connector(&self) -> Option<Box<dyn MetricsConnector>> {
    //     None
    // }

    async fn initialize(&mut self, runtime: &mut Runtime) -> Result<()>;

    async fn on_start(&mut self, runtime: &Runtime) -> Result<()>;
}

#[allow(clippy::module_name_repetitions)]
pub trait ExtensionFactory: Send + Sync {
    fn create(&self) -> Box<dyn Extension>;
}
