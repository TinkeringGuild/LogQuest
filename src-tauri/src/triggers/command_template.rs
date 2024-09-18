use super::template_string::TemplateString;
use crate::common::security::{is_crypto_available, sign, verify};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
pub struct CommandTemplate {
  pub command: String,
  pub params: Vec<TemplateString>,
  pub write_to_stdin: Option<TemplateString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[serde(tag = "variant", content = "value")]
#[ts(tag = "variant", content = "value")]
pub enum CommandTemplateSecurityCheck {
  Unapproved(CommandTemplate),
  Approved(String, CommandTemplate),
}

impl CommandTemplateSecurityCheck {
  pub fn security_check(self) -> Self {
    match self {
      Self::Approved(sig, cmd_tmpl) => {
        if is_crypto_available() {
          if verify(&cmd_tmpl.format_for_security_check(), &sig) {
            Self::Approved(sig, cmd_tmpl)
          } else {
            warn!("Security check failed for CommandTemplate! Marking CommandTemplate as Unapproved:\n{cmd_tmpl:#?}");
            Self::Unapproved(cmd_tmpl)
          }
        } else {
          warn!("Unable to use the cryptographic verification system! Marking CommandTemplate as Unapproved:\n{cmd_tmpl:#?}");
          Self::Unapproved(cmd_tmpl)
        }
      }
      unapproved @ Self::Unapproved(..) => unapproved,
    }
  }
}

impl CommandTemplate {
  pub fn format_for_security_check(&self) -> String {
    let formatted_params = self
      .params
      .iter()
      .map(|p| p.tmpl())
      .collect::<Vec<&str>>()
      .join("\n");

    let formatted_input = self
      .write_to_stdin
      .as_ref()
      .map_or_else(|| String::new(), |tmpl| format!("\n\n{}", tmpl.tmpl()));

    format!("{}\n\n{formatted_params}{formatted_input}", self.command)
  }

  pub fn approve(self) -> CommandTemplateSecurityCheck {
    let sig = sign(&self.format_for_security_check());
    CommandTemplateSecurityCheck::Approved(sig, self)
  }
}
