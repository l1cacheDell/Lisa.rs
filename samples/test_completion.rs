use rig::{completion::Prompt, providers::openai::Client};

pub async fn create_completion(openai_api_key: &str, base_url: &str, model_name: &str, prompt: &str) -> String {
    let openai_client = Client::from_url(openai_api_key, base_url);
    let qwen = openai_client.agent(&model_name).build();

    let response = qwen.prompt(prompt).await.unwrap_or("Error: No response".to_string());

    response
}
