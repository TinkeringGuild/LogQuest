use super::{EffectError, EffectResult, ReadyEffect};
use crate::{
  common::bytes_to_utf8_with_escaped_special_chars, reactor::EventContext,
  triggers::command_template::CommandTemplateSecurityCheck,
};
use async_trait::async_trait;
use std::process::Stdio;
use std::sync::Arc;
use tauri::async_runtime::spawn;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::{error, info};

pub(super) struct SystemCommandEffect {
  pub(super) cmd_tmpl_sec_check: CommandTemplateSecurityCheck,
  pub(super) non_blocking: bool,
}

#[async_trait]
impl ReadyEffect for SystemCommandEffect {
  async fn fire(self: Box<Self>, context: Arc<EventContext>) -> EffectResult {
    let command_template = match self.cmd_tmpl_sec_check.security_check() {
      CommandTemplateSecurityCheck::Approved(_, cmd_tmpl) => cmd_tmpl,
      CommandTemplateSecurityCheck::Unapproved(cmd_tmpl) => {
        // Data that is loaded from the filesystem (deserialization or import) should go
        // through the `security_check()` process, so we shouldn't ever get here, but this
        // is an important last line of defense in case any other `security_check()` logic
        // was faulty.
        return Err(EffectError::CommandSecurityCheckFail(cmd_tmpl));
      }
    };

    let command_name = command_template.command;

    let args: Vec<String> = command_template
      .params
      .into_iter()
      .map(|p| p.render(&context.match_context))
      .collect();

    let formatted_command: String = std::iter::once(command_name.clone())
      .chain(args.clone().into_iter())
      .collect::<Vec<String>>()
      .join(" ");

    let write_to_stdin = command_template
      .write_to_stdin
      .map(|tmpl| tmpl.render(&context.match_context));

    let mut command = Command::new(command_name);

    command.args(&args);
    command.stdout(Stdio::null());
    command.stderr(Stdio::piped());
    command.stdin(
      write_to_stdin
        .as_ref()
        .map_or_else(|| Stdio::null(), |_| Stdio::piped()),
    );

    info!("SystemCommandEffect executing: `{formatted_command}`");

    let mut subprocess = command
      .spawn()
      .map_err(|e| EffectError::CommandIOError(e))?;

    let finish_subprocess = async move {
      if let Some(input) = write_to_stdin {
        let Some(mut stdin) = subprocess.stdin.take() else {
          return Err(EffectError::CommandStdinClosedError(formatted_command));
        };
        stdin.write_all(&input.into_bytes()).await?;
        // stdin gets dropped here, closing the pipe
      }

      match subprocess.wait_with_output().await {
        Err(io_error) => Err(EffectError::CommandIOError(io_error)),
        Ok(output) => {
          let stderr_output = bytes_to_utf8_with_escaped_special_chars(output.stderr.trim_ascii());
          match (output.status.code(), stderr_output.lines().count()) {
            (Some(0), _) => Ok(()),
            (Some(error_code), 0) => {
              error!("Command `{formatted_command}` FAILED with status code {error_code}");
              Err(EffectError::CommandFailure(formatted_command, error_code))
            }
            (Some(error_code), 1) => {
              error!("Command `{formatted_command}` FAILED with status code {error_code} and wrote to STDERR: `{stderr_output}`");
              Err(EffectError::CommandFailure(formatted_command, error_code))
            }
            (Some(error_code), 2..) => {
              error!("Command `{formatted_command}` FAILED with status code {error_code} and wrote lines to STDERR:\n{stderr_output}");
              Err(EffectError::CommandFailure(formatted_command, error_code))
            }
            (None, _) => {
              error!("Command `{formatted_command}` CRASHED or was killed by a signal");
              Err(EffectError::CommandDied(formatted_command))
            }
          }
        }
      }
    };

    if self.non_blocking {
      spawn(finish_subprocess);
      Ok(())
    } else {
      finish_subprocess.await
    }
  }
}
