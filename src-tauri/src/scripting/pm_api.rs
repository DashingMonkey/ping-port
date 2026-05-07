use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PmVariables {
    #[serde(default)]
    pub data: HashMap<String, String>,
}

impl PmVariables {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    pub fn unset(&mut self, key: &str) {
        self.data.remove(key);
    }

    /// Parse a key-value string and set the variable.
    /// Expected format for value: "quoted string" or unquoted value
    pub fn set_from_string(&mut self, key: &str, value: &str) {
        let clean_value = Self::extract_string_value(value);
        self.set(key.to_string(), clean_value);
    }

    /// Extract a string value from quoted or unquoted input
    fn extract_string_value(input: &str) -> String {
        let trimmed = input.trim();
        if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            trimmed[1..trimmed.len() - 1].to_string()
        } else {
            trimmed.to_string()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmRequest {
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub query_params: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmResponse {
    pub status: u16,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub body: Option<String>,
    pub time: u64,
}

impl PmResponse {
    /// Parse body as JSON, returning None if parsing fails
    pub fn json(&self) -> Option<serde_json::Value> {
        self.body.as_ref()?.parse::<serde_json::Value>().ok()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub error: Option<String>,
    pub duration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmApi {
    #[serde(default)]
    pub variables: PmVariables,
    #[serde(default)]
    pub request: Option<PmRequest>,
    #[serde(default)]
    pub response: Option<PmResponse>,
    #[serde(default)]
    pub test_results: Vec<TestResult>,
    #[serde(default)]
    pub console_logs: Vec<String>,
}

impl Default for PmApi {
    fn default() -> Self {
        Self {
            variables: PmVariables::new(),
            request: None,
            response: None,
            test_results: Vec::new(),
            console_logs: Vec::new(),
        }
    }
}

impl PmApi {
    /// Helper to set a variable from key and value strings
    pub fn set_variable(&mut self, key: &str, value: &str) {
        self.variables.set_from_string(key, value);
    }

    /// Helper to get a variable value
    pub fn get_variable(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Add a test result
    pub fn add_test_result(&mut self, result: TestResult) {
        self.test_results.push(result);
    }

    /// Get all test results
    pub fn get_test_results(&self) -> &[TestResult] {
        &self.test_results
    }

    /// Clear all test results
    pub fn clear_test_results(&mut self) {
        self.test_results.clear();
    }
}
