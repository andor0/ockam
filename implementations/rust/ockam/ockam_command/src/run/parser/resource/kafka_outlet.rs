use async_trait::async_trait;
use miette::{miette, Result};
use ockam_api::colors::color_primary;
use serde::{Deserialize, Serialize};

use crate::kafka::outlet;
use crate::kafka::outlet::create::CreateCommand;
use crate::run::parser::building_blocks::{ArgsToCommands, UnnamedResources};
use crate::run::parser::resource::traits::CommandsParser;
use crate::run::parser::resource::utils::parse_cmd_from_args;
use crate::run::parser::resource::ValuesOverrides;
use crate::{Command, OckamSubcommand};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KafkaOutlet {
    #[serde(alias = "kafka-outlet")]
    pub kafka_outlet: Option<UnnamedResources>,
}

impl KafkaOutlet {
    fn get_subcommand(args: &[String]) -> Result<CreateCommand> {
        if let OckamSubcommand::KafkaOutlet(cmd) = parse_cmd_from_args(CreateCommand::NAME, args)? {
            #[allow(irrefutable_let_patterns)]
            if let outlet::KafkaOutletSubcommand::Create(c) = cmd.subcommand {
                return Ok(c);
            }
        }
        Err(miette!(format!(
            "Failed to parse {} command",
            color_primary(CreateCommand::NAME)
        )))
    }
}

#[async_trait]
impl CommandsParser<CreateCommand> for KafkaOutlet {
    fn parse_commands(self, overrides: &ValuesOverrides) -> Result<Vec<CreateCommand>> {
        match self.kafka_outlet {
            Some(c) => {
                let mut cmds = c.into_commands(Self::get_subcommand)?;
                if let Some(node_name) = overrides.override_node_name.as_ref() {
                    for cmd in cmds.iter_mut() {
                        cmd.node_opts.at_node = Some(node_name.clone())
                    }
                }
                Ok(cmds)
            }
            None => Ok(vec![]),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn kafka_outlet_config() {
        let named = r#"
            kafka-outlet:
              bootstrap-server: 192.168.0.100:9092
              at: node_name
        "#;
        let parsed: KafkaOutlet = serde_yaml::from_str(named).unwrap();
        let cmds = parsed.parse_commands(&ValuesOverrides::default()).unwrap();
        assert_eq!(cmds.len(), 1);
        assert_eq!(
            cmds[0].bootstrap_server,
            SocketAddr::from_str("192.168.0.100:9092").unwrap()
        );
        assert_eq!(cmds[0].node_opts.at_node.as_ref().unwrap(), "node_name");
    }
}