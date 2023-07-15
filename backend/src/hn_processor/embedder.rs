//use std::io::Error;
//use std::{collections::HashMap, sync::Arc};

use crate::triton::{
    self, grpc_inference_service_client::GrpcInferenceServiceClient,
    model_infer_request::InferInputTensor,
};
use tokenizers::tokenizer::Tokenizer;
//use tokio::sync::Semaphore;
use tonic::{transport::Channel, Request, Status};

pub struct E5Embedder {
    client: GrpcInferenceServiceClient<Channel>,
    /// Caps concurrent requests from client
    /// Max length of document to be embedded
    max_seq_length: usize,
    tokenizer: Tokenizer,
}

#[derive(Debug)]
pub enum E5Error {
    ConnectError(String),
    TokenizerError(String),
    ParsingError(String),
}

impl From<tonic::transport::Error> for E5Error {
    fn from(err: tonic::transport::Error) -> Self {
        E5Error::ConnectError(err.to_string())
    }
}

impl From<Status> for E5Error {
    fn from(err: Status) -> Self {
        E5Error::ConnectError(err.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for E5Error {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        E5Error::TokenizerError(err.to_string())
    }
}

impl E5Embedder {
    /// Initializes embedder for e5 small.
    /// TODO: Modularize this if need be!
    pub async fn new(url: &str) -> Result<Self, E5Error> {
        let client = GrpcInferenceServiceClient::connect(url.to_owned()).await?;
        //    let semaphore = Arc::new(Semaphore::new(max_concurrent_requests));
        Ok(Self {
            client,
            max_seq_length: 512,
            tokenizer: Tokenizer::from_pretrained("intfloat/e5-small-v2", None)?,
        })
    }

    /// API will be changed, just a dummy to get comms w/ triton working
    pub async fn encode(&self, txt: &str) -> Result<Vec<Vec<i32>>, E5Error> {
        if txt.len() > self.max_seq_length {
            return Err(E5Error::TokenizerError(
                "Input sequence is too long!".into(),
            ));
        }
        let encoding = self.tokenizer.encode(txt, false)?;
        let mut client = self.client.clone();
        let request = Request::new(triton::ModelInferRequest {
            model_name: "e5-small-v2".into(),
            model_version: "1".into(),
            inputs: vec![InferInputTensor {
                name: "Single embed test".into(),
                datatype: "INT32".into(),
                contents: Some(triton::InferTensorContents {
                    int_contents: encoding.get_ids().iter().map(|x| *x as i32).collect(),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            ..Default::default()
        });
        let raw_outputs = client.model_infer(request).await?.into_inner().outputs;
        let embeddings: Result<Vec<Vec<i32>>, E5Error> = raw_outputs
            .into_iter()
            .map(|o| {
                o.contents
                    .ok_or(E5Error::ParsingError("No output tensor".into()))
                    .map(|c| c.int_contents)
            })
            .collect();

        let embeddings = embeddings?;

        Ok(embeddings)
    }
}
