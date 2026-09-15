//! Types and clients to manage node templates in an EVE-NG instance.

use crate::nodes::NodeType;
use crate::utils::{WireMap, map_or_seq, number_from_string};
use crate::{Client, Result};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// A client to manage node templates.
pub struct TemplatesClient {
    client: Client,
}

impl TemplatesClient {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Lists available node templates.
    pub async fn list(&self) -> Result<HashMap<String, String>> {
        Ok(self
            .client
            .get::<WireMap<String, String>>("list/templates/")
            .await?
            .into_data()?
            .0)
    }
}

/// Type to describe a node template.
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeTemplate {
    /// Name of the template.
    #[serde(skip)]
    pub name: String,
    /// Description of the template.
    pub description: String,
    /// List of options for the template.
    pub options: TemplateOptions,
    /// Type of node the template creates.
    #[serde(rename = "type")]
    pub node_type: NodeType,
}

impl NodeTemplate {
    pub(crate) fn defaults_map(&self) -> Result<Map<String, Value>> {
        let defaults = serde_json::to_value(&self.options)?;
        let map = defaults.as_object().unwrap();

        Ok(map
            .iter()
            .map(|(k, v)| (k.clone(), v.get("value").cloned().unwrap_or_default()))
            .collect())
    }
}

/// Type to describe the list of options in a [`NodeTemplate`].
#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateOptions {
    /// Startup config status of the node.
    pub config: ListOption<u8>,
    /// Seconds to wait before starting the node.
    pub delay: InputOption<u32>,
    /// Icon used to display the node in the lab.
    pub icon: ListOption<String>,
    /// Name used to display the node in the lab.
    pub name: InputOption<String>,
    /// Number of configured CPUs on the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<InputOption<u32>>,
    /// CPU limit status of the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpulimit: Option<CheckboxOption>,
    /// Type of console of the node.
    pub console: Option<ListOption>,
    /// Number of configured Ethernet interfaces/portgroups (Docker, Dynamips,
    /// IOL and QEMU).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethernet: Option<InputOption<u32>>,
    /// Idle PC for the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idlepc: Option<InputOption<String>>,
    /// Image used to boot the node.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<ListOption<String>>,
    /// NVRAM configured on the node, in kilobytes (IOL and Dynamips).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nvram: Option<InputOption<u32>>,
    /// QEMU version used to boot the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_version: Option<ListOption>,
    /// QEMU target architecture used to boot the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_arch: Option<ListOption>,
    /// QEMU NIC model used for the node's interfaces (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_nic: Option<ListOption>,
    /// Custom options passed to QEMU when creating the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_options: Option<InputOption<String>>,
    /// RAM configured on the node, in megabytes (Docker, Dynamips, IOL, QEMU).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram: Option<InputOption<u32>>,
    /// Number of configured Serial portgroups (IOL only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<InputOption<u32>>,
    /// Module configured in slot 1 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot1: Option<ListOption>,
    /// Module configured in slot 2 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot2: Option<ListOption>,
    /// Module configured in slot 3 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot3: Option<ListOption>,
    /// Module configured in slot 4 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot4: Option<ListOption>,
    /// Module configured in slot 5 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot5: Option<ListOption>,
    /// Module configured in slot 6 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot6: Option<ListOption>,
    /// UUID configured for the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<InputOption<String>>,
}

/// Type to describe a template option containing a list of possible values
/// a user could choose from.
#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = "
    K: Eq + std::hash::Hash + std::str::FromStr + serde::Deserialize<'de>,
    K::Err: std::fmt::Display
"))]
pub struct ListOption<K = String> {
    /// Name of the option.
    pub name: String,
    /// List of possible values.
    #[serde(deserialize_with = "map_or_seq")]
    pub list: HashMap<K, String>,
    /// Type of the option.
    #[serde(rename = "type")]
    pub option_type: String,
    /// Default value of the option.
    #[serde(deserialize_with = "number_from_string")]
    pub value: K,
}

/// Type to describe a template option that takes text input from the user.
#[derive(Debug, Serialize, Deserialize)]
pub struct InputOption<T> {
    /// Name of the option.
    pub name: String,
    /// Type of the option.
    #[serde(rename = "type")]
    pub option_type: String,
    /// Default value of the option.
    pub value: T,
}

/// Type to describe a template option that takes a boolean value.
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckboxOption {
    /// Name of the option.
    pub name: String,
    /// Type of the option.
    #[serde(rename = "type")]
    pub option_type: String,
    /// Default value of the option.
    pub value: u8,
}

/// A client to manage a single node template.
pub struct TemplateClient {
    client: Client,
    name: String,
}

impl TemplateClient {
    pub(crate) fn new(client: Client, name: impl Into<String>) -> Self {
        Self {
            client,
            name: name.into(),
        }
    }

    /// Gets the template's details.
    pub async fn get(&self) -> Result<NodeTemplate> {
        let mut resp: NodeTemplate = self
            .client
            .get(&format!("list/templates/{}", self.name))
            .await?
            .into_data()?;
        resp.name = self.name.clone();

        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn template() -> NodeTemplate {
        let json = r#"
        {
            "options": {
                "image":{"name":"Image","type":"list","list":{"vios-15":"vios-15"},"value":"vios-15"},
                "name":{"name":"Name\/prefix","type":"input","value":"vIOS"},
                "icon":{"name":"Icon","type":"list","value":"Router-2D-Gen-White-S.svg","list":{"Router-2D-Gen-White-S.svg":"Router-2D-Gen-White-S.svg","Router.png":"Router.png"}},
                "uuid":{"name":"UUID","type":"input","value":""},
                "cpulimit":{"name":"CPU Limit","type":"checkbox","value":1},
                "cpu":{"name":"CPU","type":"input","value":1},
                "ram":{"name":"RAM","type":"input","value":1024},
                "ethernet":{"name":"Ethernets","type":"input","value":4},
                "qemu_version":{"name":"QEMU Version","type":"list","value":"2.4.0","list":{"1.3.1":"1.3.1","2.0.2":"2.0.2","2.2.0":"2.2.0","2.4.0":"2.4.0","2.5.0":"2.5.0","2.6.2":"2.6.2","2.12.0":"2.12.0","3.1.0":"3.1.0","4.1.0":"4.1.0","5.2.0":"5.2.0","6.0.0":"6.0.0","":"tpl(2.4.0)"}},
                "qemu_arch":{"name":"QEMU Arch","type":"list","value":"x86_64","list":{"i386":"i386","x86_64":"x86_64","":"tpl(x86_64)"}},
                "qemu_nic":{"name":"QEMU Nic","type":"list","value":"","list":{"virtio-net-pci":"virtio-net-pci","e1000":"e1000","i82559er":"i82559er","rtl8139":"rtl8139","e1000-82545em":"e1000-82545em","vmxnet3":"vmxnet3","":"tpl(e1000)"}},
                "qemu_options":{"name":"QEMU custom options","type":"input","value":"-machine type=pc,accel=kvm -serial mon:stdio -nographic -no-user-config -nodefaults -rtc base=utc -cpu host"},
                "config":{"name":"Startup configuration","type":"list","value":"0","list":["None","Exported"]},
                "delay":{"name":"Delay (s)","type":"input","value":0},
                "console":{"name":"Console","type":"list","value":"telnet","list":{"telnet":"telnet","vnc":"vnc","rdp":"rdp"}}
            },
            "description":"Cisco vIOS Router",
            "type":"qemu",
            "qemu":{"arch":"x86_64","version":"2.4.0","options":"-machine type=pc,accel=kvm -serial mon:stdio -nographic -no-user-config -nodefaults -rtc base=utc -cpu host"}
        }
        "#;
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn deserialize_list_option() {
        let opts = template().options;
        let icon = opts.icon;

        assert_eq!(icon.value, "Router-2D-Gen-White-S.svg");
        assert_eq!(icon.option_type, "list");
        assert_eq!(icon.list.get("Router.png").unwrap(), "Router.png");
    }

    #[test]
    fn deserialize_input_option() {
        let opts = template().options;
        let ram = opts.ram.unwrap();

        assert_eq!(ram.value, 1024);
        assert_eq!(ram.option_type, "input");
        assert_eq!(ram.name, "RAM");
    }

    #[test]
    fn deserialize_checkbox_option() {
        let opts = template().options;
        let cpulimit = opts.cpulimit.unwrap();

        assert_eq!(cpulimit.value, 1);
        assert_eq!(cpulimit.option_type, "checkbox");
        assert_eq!(cpulimit.name, "CPU Limit");
    }

    #[test]
    fn defaults_map() {
        let map = template().defaults_map().unwrap();

        assert_eq!(map.get("image").unwrap(), &json!("vios-15"));
        assert_eq!(map.get("name").unwrap(), &json!("vIOS"));
        assert_eq!(
            map.get("icon").unwrap(),
            &json!("Router-2D-Gen-White-S.svg")
        );
        assert_eq!(map.get("uuid").unwrap(), &json!(""));
        assert_eq!(map.get("cpu").unwrap(), &json!(1));
        assert_eq!(map.get("ram").unwrap(), &json!(1024));
        assert_eq!(map.get("ethernet").unwrap(), &json!(4));
        assert_eq!(map.get("qemu_version").unwrap(), &json!("2.4.0"));
        assert_eq!(map.get("qemu_arch").unwrap(), &json!("x86_64"));
        assert_eq!(map.get("qemu_nic").unwrap(), &json!(""));
        assert_eq!(
            map.get("qemu_options").unwrap(),
            &json!(
                "-machine type=pc,accel=kvm -serial mon:stdio -nographic -no-user-config -nodefaults -rtc base=utc -cpu host"
            )
        );
        assert_eq!(map.get("config").unwrap(), &json!(0));
        assert_eq!(map.get("delay").unwrap(), &json!(0));
        assert_eq!(map.get("console").unwrap(), &json!("telnet"));
    }
}
