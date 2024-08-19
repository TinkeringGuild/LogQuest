use super::template_string::TemplateString;
use crate::common::security::{is_crypto_available, verify};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
pub struct CommandTemplate {
  pub command: String,
  pub params: Vec<TemplateString>,
  pub write_to_stdin: Option<TemplateString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
pub enum CommandTemplateSecurityCheck {
  Unapproved(CommandTemplate),
  Approved(String, CommandTemplate),
}

impl CommandTemplateSecurityCheck {
  pub fn security_check(self) -> Self {
    match self {
      unapproved @ Self::Unapproved(_) => unapproved,
      Self::Approved(sig, cmd_tmpl) => {
        if is_crypto_available() {
          if verify(&cmd_tmpl.format_for_security_check(), &sig) {
            Self::Approved(sig, cmd_tmpl)
          } else {
            Self::Unapproved(cmd_tmpl)
          }
        } else {
          warn!("Unable to use the cryptographic verification system for a CommandTemplate - discarding Signature");
          Self::Unapproved(cmd_tmpl)
        }
      }
    }
  }
}

impl CommandTemplate {
  pub fn format_for_security_check(&self) -> String {
    let formatted_params = self
      .params
      .iter()
      .map(|p| p.template())
      .collect::<Vec<&str>>()
      .join("\n");

    let formatted_input = self
      .write_to_stdin
      .as_ref()
      .map_or_else(|| String::new(), |tmpl| format!("\n\n{}", tmpl.template()));

    format!("{}\n\n{formatted_params}{formatted_input}", self.command)
  }
}
