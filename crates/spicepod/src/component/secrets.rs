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

use serde::{Deserialize, Serialize};

/// The secrets configuration for a Spicepod.
///
/// Example:
/// ```yaml
/// secrets:
///   store: file
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Secrets {
    pub store: SpiceSecretStore,
}

impl Default for Secrets {
    fn default() -> Self {
        Self {
            store: SpiceSecretStore::File,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SpiceSecretStore {
    File,
    Env,
    Kubernetes,
    Keyring,
    #[serde(rename = "aws_secrets_manager")]
    AwsSecretsManager,
}
