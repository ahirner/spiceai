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
#![allow(clippy::missing_errors_doc)]
use crate::embeddings::{Embed, EmbeddingInput, Error as EmbedError, Result as EmbedResult};
use crate::nql::{Error as NqlError, Nql, Result as NqlResult};

use async_openai::types::CreateEmbeddingRequestArgs;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionResponseFormat, ChatCompletionResponseFormatType,
        CreateChatCompletionRequestArgs, EmbeddingInput as OpenAiEmbeddingInput,
    },
    Client,
};
use async_trait::async_trait;
use serde_json::Value;
use snafu::ResultExt;

const MAX_COMPLETION_TOKENS: u16 = 1024_u16; // Avoid accidentally using infinite tokens. Should think about this more.

pub(crate) const GPT3_5_TURBO_INSTRUCT: &str = "gpt-3.5-turbo";
pub(crate) const TEXT_EMBED_3_SMALL: &str = "text-embedding-3-small";

pub const DEFAULT_LLM_MODEL: &str = GPT3_5_TURBO_INSTRUCT;
pub const DEFAULT_EMBEDDING_MODEL: &str = TEXT_EMBED_3_SMALL;

pub struct Openai {
    client: Client<OpenAIConfig>,
    model: String,
}

impl Default for Openai {
    fn default() -> Self {
        Self::new(DEFAULT_LLM_MODEL.to_string(), None, None, None, None)
    }
}

impl Openai {
    #[must_use]
    pub fn new(
        model: String,
        api_base: Option<String>,
        api_key: Option<String>,
        org_id: Option<String>,
        project_id: Option<String>,
    ) -> Self {
        let mut cfg = OpenAIConfig::new()
            .with_org_id(org_id.unwrap_or_default())
            .with_project_id(project_id.unwrap_or_default());

        // If an API key is provided, use it. Otherwise use default from env variables.
        if let Some(api_key) = api_key {
            cfg = cfg.with_api_key(api_key);
        }
        if let Some(api_base) = api_base {
            cfg = cfg.with_api_base(api_base);
        }
        Self {
            client: Client::with_config(cfg),
            model,
        }
    }

    /// Convert the Json object returned when using a `{ "type": "json_object" } ` response format.
    /// Expected format is `"content": "{\"arbitrary_key\": \"arbitrary_value\"}"`
    pub fn convert_json_object_to_sql(raw_json: &str) -> NqlResult<Option<String>> {
        let result: Value = serde_json::from_str(raw_json)
            .boxed()
            .map_err(|source| NqlError::FailedToLoadModel { source })?;
        Ok(result["sql"].as_str().map(std::string::ToString::to_string))
    }
}

#[async_trait]
impl Nql for Openai {
    async fn run(&mut self, prompt: String) -> NqlResult<Option<String>> {
        let messages: Vec<ChatCompletionRequestMessage> = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content("Return JSON, with the requested SQL under 'sql'.")
                .build()
                .boxed()
                .map_err(|source| NqlError::FailedToLoadTokenizer { source })?
                .into(),
            ChatCompletionRequestSystemMessageArgs::default()
                .content(prompt)
                .build()
                .boxed()
                .map_err(|source| NqlError::FailedToLoadTokenizer { source })?
                .into(),
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model(self.model.clone())
            .response_format(ChatCompletionResponseFormat {
                r#type: ChatCompletionResponseFormatType::JsonObject,
            })
            .messages(messages)
            .max_tokens(MAX_COMPLETION_TOKENS)
            .build()
            .boxed()
            .map_err(|source| NqlError::FailedToLoadModel { source })?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .boxed()
            .map_err(|source| NqlError::FailedToRunModel { source })?;

        if let Some(usage) = response.usage {
            if usage.completion_tokens >= u32::from(MAX_COMPLETION_TOKENS) {
                tracing::warn!(
                    "Completion response may have been cut off after {} tokens",
                    MAX_COMPLETION_TOKENS
                );
            }
        }

        match response
            .choices
            .iter()
            .find_map(|c| c.message.content.clone())
        {
            Some(json_resp) => Self::convert_json_object_to_sql(&json_resp),
            None => Ok(None),
        }
    }
}

#[async_trait]
impl Embed for Openai {
    async fn embed(&mut self, input: EmbeddingInput) -> EmbedResult<Vec<Vec<f32>>> {
        let req = CreateEmbeddingRequestArgs::default()
            .model(self.model.clone())
            .input(to_openai_embedding_input(input))
            .build()
            .boxed()
            .map_err(|source| EmbedError::FailedToPrepareInput { source })?;

        let embedding: Vec<Vec<f32>> = self
            .client
            .embeddings()
            .create(req)
            .await
            .boxed()
            .map_err(|source| EmbedError::FailedToCreateEmbedding { source })?
            .data
            .iter()
            .map(|d| d.embedding.clone())
            .collect();

        Ok(embedding)
    }
}

fn to_openai_embedding_input(input: EmbeddingInput) -> OpenAiEmbeddingInput {
    match input {
        EmbeddingInput::String(s) => OpenAiEmbeddingInput::String(s),
        EmbeddingInput::Tokens(t) => OpenAiEmbeddingInput::IntegerArray(t),
        EmbeddingInput::StringBatch(sb) => OpenAiEmbeddingInput::StringArray(sb),
        EmbeddingInput::TokensBatch(tb) => OpenAiEmbeddingInput::ArrayOfIntegerArray(tb),
    }
}
