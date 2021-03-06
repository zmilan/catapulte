use super::TemplateProviderError;
use crate::service::template::manager::{TemplateManager, TemplateManagerError};
use crate::service::template::template::Template;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::fs::{read_to_string, File};
use std::io::BufReader;
use std::path::Path;

pub const CONFIG_PROVIDER_LOCAL_ROOT: &str = "TEMPLATE_ROOT";

fn default_mjml_path() -> String {
    "template.mjml".into()
}

#[derive(Debug, Deserialize)]
pub struct LocalMetadata {
    name: String,
    description: String,
    #[serde(default = "default_mjml_path")]
    mjml: String,
    attributes: JsonValue,
}

#[derive(Clone, Debug)]
pub struct LocalTemplateProvider {
    root: String,
}

impl LocalTemplateProvider {
    pub fn from_env() -> Result<Self, TemplateProviderError> {
        Ok(Self::new(
            std::env::var(CONFIG_PROVIDER_LOCAL_ROOT)
                .unwrap_or_else(|_| String::from("./template")),
        ))
    }

    pub fn new(root: String) -> Self {
        Self { root }
    }
}

#[async_trait]
impl TemplateManager for LocalTemplateProvider {
    async fn find_by_name(&self, name: &str) -> Result<Template, TemplateManagerError> {
        info!("find_by_name: {}", name);
        let path = Path::new(self.root.as_str())
            .join(name)
            .join("metadata.json");
        let metadata_file = File::open(path)?;
        let metadata_reader = BufReader::new(metadata_file);
        let metadata: LocalMetadata = serde_json::from_reader(metadata_reader)?;
        let mjml_path = Path::new(self.root.as_str()).join(name).join(metadata.mjml);
        let content = read_to_string(mjml_path)?;
        Ok(Template {
            name: metadata.name,
            description: metadata.description,
            content,
            attributes: metadata.attributes,
        })
    }
}

// LCOV_EXCL_START
#[cfg(test)]
mod tests {
    use super::*;
    use env_test_util::TempEnvVar;

    fn get_root() -> String {
        match std::env::var(CONFIG_PROVIDER_LOCAL_ROOT) {
            Ok(value) => value,
            Err(_) => String::from("template"),
        }
    }

    #[test]
    #[serial]
    fn without_template_root() {
        let _env_base_url = TempEnvVar::new(CONFIG_PROVIDER_LOCAL_ROOT);
        let result = LocalTemplateProvider::from_env().unwrap();
        assert_eq!(result.root, "./template");
    }

    #[test]
    #[serial]
    fn with_template_root() {
        let _env_base_url = TempEnvVar::new(CONFIG_PROVIDER_LOCAL_ROOT).with("./somewhere");
        let result = LocalTemplateProvider::from_env().unwrap();
        assert_eq!(result.root, "./somewhere");
    }

    #[actix_rt::test]
    #[serial]
    async fn local_find_by_name_not_found() {
        let manager = LocalTemplateProvider::new(get_root());
        assert!(match manager.find_by_name("not_found").await.unwrap_err() {
            TemplateManagerError::TemplateNotFound => true,
            _ => false,
        });
    }

    #[actix_rt::test]
    #[serial]
    async fn local_find_by_name_success() {
        let manager = LocalTemplateProvider::new(get_root());
        let result = manager.find_by_name("user-login").await.unwrap();
        assert_eq!(result.name, "user-login");
        assert_eq!(result.description, "Template for login with magic link");
    }
}
// LCOV_EXCL_END
