/*
Copyright 2024 The Spice.ai OSS Authors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

     https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::{collections::HashMap, sync::Arc};

use datafusion::{
    error::DataFusionError,
    execution::{
        object_store::{DefaultObjectStoreRegistry, ObjectStoreRegistry},
        runtime_env::{RuntimeConfig, RuntimeEnv},
    },
};
use object_store::{aws::AmazonS3Builder, ClientOptions, ObjectStore};
use url::{form_urlencoded::parse, Url};

#[cfg(feature = "ftp")]
use crate::objectstore::ftp::FTPObjectStore;
#[cfg(feature = "ftp")]
use crate::objectstore::sftp::SFTPObjectStore;
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default)]
pub struct SpiceObjectStoreRegistry {
    inner: DefaultObjectStoreRegistry,
}

impl SpiceObjectStoreRegistry {
    #[must_use]
    pub fn new() -> Self {
        SpiceObjectStoreRegistry::default()
    }

    fn get_feature_store(url: &Url) -> datafusion::error::Result<Arc<dyn ObjectStore>> {
        {
            if url.as_str().starts_with("s3://") {
                if let Some(bucket_name) = url.host_str() {
                    let mut s3_builder = AmazonS3Builder::from_env()
                        .with_bucket_name(bucket_name)
                        .with_allow_http(true);
                    let mut client_options = ClientOptions::default();

                    let params: HashMap<String, String> =
                        parse(url.fragment().unwrap_or_default().as_bytes())
                            .into_owned()
                            .collect();

                    if let Some(region) = params.get("region") {
                        s3_builder = s3_builder.with_region(region);
                    }
                    if let Some(endpoint) = params.get("endpoint") {
                        s3_builder = s3_builder.with_endpoint(endpoint);
                    }
                    if let Some(timeout) = params.get("timeout") {
                        client_options = client_options.with_timeout(
                            fundu::parse_duration(timeout).map_err(|_| {
                                DataFusionError::Configuration(format!(
                                    "Unable to parse timeout: {timeout}",
                                ))
                            })?,
                        );
                    }
                    if let (Some(key), Some(secret)) = (params.get("key"), params.get("secret")) {
                        s3_builder = s3_builder.with_access_key_id(key);
                        s3_builder = s3_builder.with_secret_access_key(secret);
                    } else {
                        s3_builder = s3_builder.with_skip_signature(true);
                    };
                    s3_builder = s3_builder.with_client_options(client_options);

                    return Ok(Arc::new(s3_builder.build()?));
                }
            }
            #[cfg(feature = "ftp")]
            if url.as_str().starts_with("ftp://") {
                if let Some(host) = url.host() {
                    let params: HashMap<String, String> =
                        parse(url.fragment().unwrap_or_default().as_bytes())
                            .into_owned()
                            .collect();

                    let port = params
                        .get("port")
                        .map_or("21".to_string(), ToOwned::to_owned);
                    let user = params.get("user").map(ToOwned::to_owned).ok_or_else(|| {
                        DataFusionError::Configuration("No user provided for FTP".to_string())
                    })?;
                    let password =
                        params
                            .get("password")
                            .map(ToOwned::to_owned)
                            .ok_or_else(|| {
                                DataFusionError::Configuration(
                                    "No password provided for FTP".to_string(),
                                )
                            })?;

                    let ftp_object_store =
                        FTPObjectStore::new(user, password, host.to_string(), port);
                    return Ok(Arc::new(ftp_object_store) as Arc<dyn ObjectStore>);
                }
            }
            #[cfg(feature = "ftp")]
            if url.as_str().starts_with("sftp://") {
                if let Some(host) = url.host() {
                    let params: HashMap<String, String> =
                        parse(url.fragment().unwrap_or_default().as_bytes())
                            .into_owned()
                            .collect();

                    let port = params
                        .get("port")
                        .map_or("22".to_string(), ToOwned::to_owned);
                    let user = params.get("user").map(ToOwned::to_owned).ok_or_else(|| {
                        DataFusionError::Configuration("No user provided for SFTP".to_string())
                    })?;
                    let password =
                        params
                            .get("password")
                            .map(ToOwned::to_owned)
                            .ok_or_else(|| {
                                DataFusionError::Configuration(
                                    "No password provided for SFTP".to_string(),
                                )
                            })?;

                    let sftp_object_store =
                        SFTPObjectStore::new(user, password, host.to_string(), port);
                    return Ok(Arc::new(sftp_object_store) as Arc<dyn ObjectStore>);
                }
            }
        }

        Err(DataFusionError::Execution(format!(
            "No object store available for: {:?}/{}",
            url.host_str(),
            url.path(),
        )))
    }
}

impl ObjectStoreRegistry for SpiceObjectStoreRegistry {
    fn register_store(
        &self,
        url: &Url,
        store: Arc<dyn ObjectStore>,
    ) -> Option<Arc<dyn ObjectStore>> {
        self.inner.register_store(url, store)
    }

    fn get_store(&self, url: &Url) -> datafusion::error::Result<Arc<dyn ObjectStore>> {
        self.inner.get_store(url).or_else(|_| {
            let store = Self::get_feature_store(url)?;
            self.inner.register_store(url, Arc::clone(&store));

            Ok(store)
        })
    }
}

// This method uses unwrap_or_default, however it should never fail on the initialization. See
// RuntimeEnv::default()
pub(crate) fn default_runtime_env() -> Arc<RuntimeEnv> {
    Arc::new(
        RuntimeEnv::new(
            RuntimeConfig::default()
                .with_object_store_registry(Arc::new(SpiceObjectStoreRegistry::default())),
        )
        .unwrap_or_default(),
    )
}
