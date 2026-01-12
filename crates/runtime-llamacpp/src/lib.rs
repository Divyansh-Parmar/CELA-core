use async_trait::async_trait;
use lie_core::error::EngineError;
use lie_core::runtime::{InferenceOptions, ModelLoadConfig, ModelRuntime, InferenceResult, InferenceStatus, Usage};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{LlamaModel, AddBos, Special};
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use std::num::NonZeroU32;
use std::time::Instant;

pub struct LlamaCppRuntime {
    backend: LlamaBackend,
    model: Option<LlamaModel>,
}

impl LlamaCppRuntime {
    pub fn new() -> Self {
        Self {
            backend: LlamaBackend::init().unwrap(),
            model: None,
        }
    }
}

#[async_trait]
impl ModelRuntime for LlamaCppRuntime {
    async fn load(&mut self, config: &ModelLoadConfig) -> Result<(), EngineError> {
        let model_params = LlamaModelParams::default();
        let model_path_str = config.model_path.to_str()
            .ok_or_else(|| EngineError::Config("Invalid model path".to_string()))?;

        let model = LlamaModel::load_from_file(&self.backend, model_path_str, &model_params)
            .map_err(|e| EngineError::Runtime(format!("Failed to load model: {}", e)))?;

        self.model = Some(model);
        Ok(())
    }

    async fn infer(&mut self, prompt: &str, options: InferenceOptions) -> Result<InferenceResult, EngineError> {
        let start_time = Instant::now();
        let model = self.model.as_ref().ok_or(EngineError::ModelNotLoaded)?;
        
        let n_ctx_size = 2048; // TODO: Get from model or config
        
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(NonZeroU32::new(n_ctx_size).unwrap()));
            
        let mut ctx = model.new_context(&self.backend, ctx_params)
            .map_err(|e| EngineError::Runtime(format!("Failed to create context: {}", e)))?;

        // 1. Tokenize (AddBos::Always)
        let tokens_list = model.str_to_token(prompt, AddBos::Always)
            .map_err(|e| EngineError::Runtime(format!("Tokenization failed: {}", e)))?;
            
        let input_tokens_count = tokens_list.len() as u32;

        // Context Limit Check
        if input_tokens_count as u32 > n_ctx_size {
             return Err(EngineError::Runtime(format!("Input length ({}) exceeds context size ({})", input_tokens_count, n_ctx_size)));
        }

        // 2. Prepare batch
        let mut batch = LlamaBatch::new(2048, 1); 
        let last_index = (input_tokens_count as i32) - 1;
        
        for (i, token) in tokens_list.iter().enumerate() {
            let is_last = i as i32 == last_index;
            batch.add(*token, i as i32, &[0], is_last)
                .map_err(|e| EngineError::Runtime(format!("Batch add failed: {}", e)))?;
        }

        // 3. Decode
        ctx.decode(&mut batch)
            .map_err(|e| EngineError::Runtime(format!("Decode failed: {}", e)))?;

        // 4. Generation Loop
        let mut response_tokens = Vec::new();
        let max_gen_tokens = options.max_tokens.unwrap_or(128);
        let max_time_ms = options.max_time_ms.unwrap_or(30000); // 30s hard limit
        
        let mut current_pos = input_tokens_count as i32;
        let mut completion_status = InferenceStatus::Success;

        for _ in 0..max_gen_tokens {
            // Check Time Limit
            if start_time.elapsed().as_millis() as u64 > max_time_ms {
                completion_status = InferenceStatus::Truncated;
                break;
            }
            
            // Check Context Limit (Soft check, though batch/ctx might err first)
            if current_pos as u32 >= n_ctx_size {
                 completion_status = InferenceStatus::Truncated;
                 break;
            }

            let candidates = ctx.candidates_ith(batch.n_tokens() - 1);
            
            // Greedy Sampling (Logits)
            let next_token_data = candidates.max_by(|a, b| a.logit().partial_cmp(&b.logit()).unwrap_or(std::cmp::Ordering::Equal))
                .ok_or_else(|| EngineError::Runtime("No candidates found".to_string()))?;
                
            let next_token = next_token_data.id();
            
            if next_token == model.token_eos() {
                break;
            }

            response_tokens.push(next_token);

            batch.clear();
            batch.add(next_token, current_pos, &[0], true)
                 .map_err(|e| EngineError::Runtime(format!("Batch add failed in loop: {}", e)))?;
            
            current_pos += 1;

            ctx.decode(&mut batch)
                .map_err(|e| EngineError::Runtime(format!("Decode loop failed: {}", e)))?;
        }
        
        // If we hit max_gen_tokens without EOS, status is Truncated?
        // Actually, if loop finishes normally, it means we hit limit.
        // If we broke due to EOS, we are good.
        if completion_status == InferenceStatus::Success && response_tokens.len() as u32 == max_gen_tokens {
             completion_status = InferenceStatus::Truncated;
        }

        // 5. Detokenize
        let mut output_string = String::new();
        for token in response_tokens.iter() {
             let piece = model.token_to_str(*token, Special::Plaintext) 
                 .map_err(|e| EngineError::Runtime(format!("Detokenization failed: {}", e)))?;
             output_string.push_str(&piece);
        }

        let output_tokens_count = response_tokens.len() as u32;
        let total_tokens_count = input_tokens_count + output_tokens_count;
        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(InferenceResult {
            text: output_string,
            usage: Usage {
                input_tokens: input_tokens_count,
                output_tokens: output_tokens_count,
                total_tokens: total_tokens_count,
                duration_ms,
            },
            status: completion_status,
        })
    }

    async fn unload(&mut self) -> Result<(), EngineError> {
        self.model = None;
        Ok(())
    }
}
