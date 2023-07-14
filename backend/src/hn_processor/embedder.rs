use std::collections::HashMap;

use crate::triton::triton;
use tokenizers::tokenizer::{Result, Tokenizer};

pub struct Embedder {
    /// Max length of document to be embedded
    max_seq_length: usize,
    tokenizer: Tokenizer,
}

impl Embedder {
    /// Initializes embedder for e5 small.
    /// TODO: Modularize this if need be!
    pub fn new() -> Result<Self> {
        Ok(Self {
            max_seq_length: 512,
            tokenizer: Tokenizer::from_pretrained("intfloat/e5-small-v2", None)?,
        })
    }

    /// API will be changed, just a dummy to get comms w/ triton working
    pub async fn encode(&self, txt: &str) -> Result<()> {
        let encoding = self.tokenizer.encode(txt, false)?;
        let request = tonic::Request::new(triton::ModelInferRequest {
            model_name: "e5-small-v2".into(),
            model_version: "1".into(),
            id: "".into(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            raw_input_contents: Vec::new(),
            parameters: HashMap::new(),
        });
        Ok(())
    }
}
