use eveng::interfaces::InterfaceType;
use eveng::labs::{AddLabRequest, EditLabRequest, LabClient};
use eveng::networks::{AddNetworkRequest, EditNetworkRequest};
use eveng::nodes::{
    AddNodeRequest, EditNodeRequest, Node, NodeClient, NodeStatus, NodeType, StartupConfig,
};
use eveng::users::{AddUserRequest, EditUserRequest};
use eveng::{Client, Error, Result};
use std::env;
use tokio::sync::OnceCell;

#[derive(Clone)]
pub struct TestEnv {
    client: Client,
}

// Setup test environment + cleanup any stale resources from previous test runs.
static INIT: OnceCell<()> = OnceCell::const_new();

impl TestEnv {
    pub async fn setup() -> Result<Self> {
        let url = Self::url();
        let username = Self::username();
        let password = env::var("EVE_NG_PASS").unwrap_or("eve".to_string());

        let client = Client::login(url, username, password).await?;

        INIT.get_or_init(|| async {
            let _ = client.user("test").unwrap().delete().await;

            let folders = ["/New Folder", "/Test Folder", "/New Folder 1"];
            for folder in folders {
                let _ = client.folder(folder).unwrap().delete().await;
            }

            let labs = ["Test", "Test Lab"];
            for lab in labs {
                let _ = client.folder("/").unwrap().lab(lab).unwrap().delete().await;
            }

            let req = AddLabRequest::new("test").unwrap();
            let _ = client.folder("/").unwrap().labs().add(req).await;
        })
        .await;

        Ok(Self { client })
    }

    pub fn url() -> String {
        env::var("EVE_NG_URL").unwrap_or("http://localhost".to_string())
    }

    pub fn username() -> String {
        env::var("EVE_NG_USER").unwrap_or("admin".to_string())
    }

    pub async fn teardown(self) -> Result<()> {
        self.client.logout().await
    }

    pub async fn new_vpcs_node(&self, lab: &LabClient) -> Result<NodeClient> {
        let tmpl = self.client.system().node_template("vpcs").get().await?;
        let req = AddNodeRequest::vpcs(&tmpl)?;
        lab.nodes().add(req).await
    }

    pub async fn new_vios_node(&self, lab: &LabClient) -> Result<NodeClient> {
        let tmpl = self.client.system().node_template("vios").get().await?;
        let req = AddNodeRequest::qemu(&tmpl)?;
        lab.nodes().add(req).await
    }
}

#[tokio::test]
#[ignore]
async fn login_success() -> Result<()> {
    let env = TestEnv::setup().await;
    assert!(env.is_ok());

    env.unwrap().teardown().await
}

#[tokio::test]
#[ignore]
async fn login_failure() {
    let result = Client::login(TestEnv::url(), TestEnv::username(), "incorrect").await;
    assert!(matches!(result, Err(Error::Api { code: 500, .. })));
}

#[tokio::test]
#[ignore]
async fn user_lifecycle() -> Result<()> {
    let env = TestEnv::setup().await?;

    let req = AddUserRequest::new("test", "test")?
        .role("admin")
        .name("Test User")?
        .role("admin")
        .expiration(-1);
    let user = env.client.users().add(req).await?;
    let added = user.get().await?;

    user.edit(EditUserRequest::new().email("test@test.com"))
        .await?;
    let edited = user.get().await?;
    assert_eq!(edited.email, Some("test@test.com".to_string()));
    assert_ne!(edited.email, added.email);

    let deleted = user.delete().await;
    assert!(deleted.is_ok());

    env.teardown().await
}

#[tokio::test]
#[ignore]
async fn non_existent_user() -> Result<()> {
    let env = TestEnv::setup().await?;
    let client = &env.client;

    let user = client.user("test")?;

    let result = user.get().await;
    assert!(matches!(result, Err(Error::Api { .. })));

    let req = EditUserRequest::new();
    let result = user.edit(req).await;
    assert!(matches!(result, Err(Error::Api { .. })));

    let result = user.delete().await;
    assert!(matches!(result, Err(Error::Api { .. })));

    env.teardown().await
}

#[tokio::test]
#[ignore]
async fn add_existing_user() -> Result<()> {
    let env = TestEnv::setup().await?;

    let user = env
        .client
        .users()
        .add(AddUserRequest::new("test", "test")?.role("admin"))
        .await?;

    let result = env
        .client
        .users()
        .add(AddUserRequest::new("test", "test")?.role("admin"))
        .await;
    assert!(matches!(result, Err(Error::Api { code: 500, .. })));

    user.delete().await?;
    env.teardown().await
}

#[tokio::test]
#[ignore]
async fn folder_lifecycle() -> Result<()> {
    let env = TestEnv::setup().await?;
    let root = env.client.folder("/")?;

    let folder = root.add("New Folder").await?;
    let folders = root.list().await?.folders;
    assert!(folders.iter().any(|f| f.path == "/New Folder"));

    let folder = folder.rename("Test Folder").await?;
    let folders = root.list().await?.folders;
    assert!(folders.iter().any(|f| f.path == "/Test Folder"));

    let folder2 = root.add("New Folder 1").await?;
    folder2.move_to(&folder).await?;
    let folders = folder.list().await?.folders;
    assert!(
        folders
            .iter()
            .any(|f| f.path == "/Test Folder/New Folder 1")
    );

    let result = folder.delete().await;
    assert!(result.is_ok());

    env.teardown().await
}

#[tokio::test]
#[ignore]
async fn non_existent_folder() -> Result<()> {
    let env = TestEnv::setup().await?;

    let folder = env.client.folder("/New Folder")?;
    let listed = folder.list().await;
    assert!(matches!(listed, Err(Error::Api { .. })));

    let renamed = folder.rename("Test Folder").await;
    assert!(matches!(renamed, Err(Error::Api { .. })));

    let folder = env.client.folder("/New Folder")?;
    let folder2 = env.client.folder("/Test Folder")?;
    let moved = folder.move_to(&folder2).await;
    assert!(matches!(moved, Err(Error::Api { .. })));

    let folder = env.client.folder("/New Folder")?;
    let deleted = folder.delete().await;
    assert!(matches!(deleted, Err(Error::Api { .. })));

    env.teardown().await
}

#[tokio::test]
#[ignore]
async fn add_existing_folder() -> Result<()> {
    let env = TestEnv::setup().await?;
    let root = env.client.folder("/")?;

    let folder = root.add("New Folder").await?;

    let result = root.add("New Folder").await;
    assert!(matches!(result, Err(Error::Api { .. })));

    folder.delete().await?;
    env.teardown().await
}

#[tokio::test]
#[ignore]
async fn lab_lifecycle() -> Result<()> {
    let env = TestEnv::setup().await?;
    let root = env.client.folder("/")?;

    let lab = root
        .labs()
        .add(
            AddLabRequest::new("Test")?
                .author("Test User")
                .description("A test lab")
                .body("This lab is created for test purposes."),
        )
        .await?;
    let added = lab.get().await?;

    lab.edit(EditLabRequest::new().version(2).clear_description())
        .await?;
    let edited = lab.get().await?;
    assert_ne!(edited.version, added.version);
    assert_ne!(edited.description, added.description);

    let lab = lab.rename("Test Lab").await?;
    let renamed = lab.get().await?;
    assert_eq!(renamed.name, "Test Lab".to_string());

    let folder = root.add("New Folder").await?;
    let lab = lab.move_to(&folder).await?;
    let moved = folder.list().await?.labs;
    assert!(moved.iter().any(|f| f.filename == "Test Lab.unl"));

    let deleted = lab.delete().await;
    assert!(deleted.is_ok());

    folder.delete().await?;
    env.teardown().await
}

#[tokio::test]
#[ignore]
async fn non_existent_lab() -> Result<()> {
    let env = TestEnv::setup().await?;
    let lab = env.client.folder("/")?.lab("Test")?;

    let result = lab.get().await;
    assert!(matches!(result, Err(Error::Api { .. })));

    let edited = lab.edit(EditLabRequest::new()).await;
    assert!(matches!(edited, Err(Error::Api { .. })));

    let renamed = lab.rename("Test").await;
    assert!(matches!(renamed, Err(Error::Api { .. })));

    let lab = env.client.folder("/")?.lab("Test")?;
    let folder = env.client.folder("/Test Folder")?;
    let moved = lab.move_to(&folder).await;
    assert!(matches!(moved, Err(Error::Api { .. })));

    let lab = env.client.folder("/")?.lab("Test")?;
    let deleted = lab.delete().await;
    assert!(matches!(deleted, Err(Error::Api { .. })));

    env.teardown().await
}

#[tokio::test]
#[ignore]
async fn add_existing_lab() -> Result<()> {
    let env = TestEnv::setup().await?;
    let root = env.client.folder("/")?;

    let lab = root.labs().add(AddLabRequest::new("Test")?).await?;

    let result = root.labs().add(AddLabRequest::new("Test")?).await;
    assert!(matches!(result, Err(Error::Api { .. })));

    lab.delete().await?;
    env.teardown().await
}

#[tokio::test]
#[ignore]
async fn node_lifecycle() -> Result<()> {
    let env = TestEnv::setup().await?;
    let lab = env.client.folder("/")?.lab("test")?;

    let template = env.client.system().node_template("veos").get().await?;
    let req = AddNodeRequest::qemu(&template)?
        .position(100, 100)
        .cpu(1)
        .ram(2048)
        .ethernet(2);
    let node = lab.nodes().add(req).await?;
    let added = node.get().await?;
    assert_eq!(added.left, 100);
    assert_eq!(added.top, 100);
    assert!(matches!(added.node_type, NodeType::Qemu));
    assert_eq!(added.template, "veos".to_string());
    assert_eq!(added.cpu, Some(1));
    assert_eq!(added.ram, Some(2048));
    assert_eq!(added.ethernet, Some(2));

    let req = EditNodeRequest::qemu(&added)?.cpu(2);
    node.edit(req).await?;
    let edited = node.get().await?;
    assert_eq!(edited.cpu, Some(2));

    node.start().await?;
    assert!(matches!(node.status().await?, NodeStatus::Running));
    assert_eq!(
        env.client.system().auth_status().await?.lab,
        Some("/test.unl".to_string())
    );

    node.stop().await?;
    assert!(matches!(node.status().await?, NodeStatus::Stopped));

    let deleted = node.delete().await;
    assert!(deleted.is_ok());

    env.teardown().await
}

#[tokio::test]
#[ignore]
async fn non_existent_node() -> Result<()> {
    let env = TestEnv::setup().await?;
    let lab = env.client.folder("/")?.lab("test")?;
    let node = lab.node(20);

    let result = node.get().await;
    assert!(matches!(result, Err(Error::Api { .. })));

    let sample_node: Node = serde_json::from_value(serde_json::json!({
        "config": StartupConfig::None,
        "delay": 0,
        "icon": "Router.svg",
        "left": 0,
        "name": "dummy",
        "type": NodeType::Vpcs,
        "status": NodeStatus::Stopped,
        "template": "vpcs",
        "top": 0,
        "url": "",
        "console": "",
        "image": "",
    }))?;
    let req = EditNodeRequest::vpcs(&sample_node)?.position(100, 100);
    let edited = node.edit(req).await;
    assert!(matches!(edited, Err(Error::Api { .. })));

    let deleted = node.delete().await;
    assert!(matches!(deleted, Err(Error::Api { .. })));

    env.teardown().await
}

#[tokio::test]
#[ignore]
async fn network_lifecycle() -> Result<()> {
    let env = TestEnv::setup().await?;
    let lab = env.client.folder("/")?.lab("test")?;

    let req = AddNetworkRequest::new("pnet0");
    let network = lab.networks().add(req).await?;
    let added = network.get().await?;

    network
        .edit(EditNetworkRequest::new().network_type("bridge"))
        .await?;
    let edited = network.get().await?;
    assert_ne!(edited.network_type, added.network_type);
    assert_eq!(edited.id, added.id);

    let deleted = network.delete().await;
    assert!(deleted.is_ok());

    env.teardown().await
}

#[tokio::test]
#[ignore]
async fn non_existent_network() -> Result<()> {
    let env = TestEnv::setup().await?;
    let lab = env.client.folder("/")?.lab("test")?;
    let network = lab.network(10);

    let result = network.get().await;
    assert!(matches!(result, Err(Error::Api { .. })));

    let req = EditNetworkRequest::new();
    let edited = network.edit(req).await;
    assert!(matches!(edited, Err(Error::Api { .. })));

    let deleted = network.delete().await;
    assert!(matches!(deleted, Err(Error::Api { .. })));

    env.teardown().await
}

#[tokio::test]
#[ignore]
async fn non_existent_interface() -> Result<()> {
    let env = TestEnv::setup().await?;
    let lab = env.client.folder("/")?.lab("test")?;

    let tmpl = env.client.system().node_template("vpcs").get().await?;
    let node = lab.nodes().add(AddNodeRequest::vpcs(&tmpl)?).await?;
    let iface = node.ethernet(1);

    let result = iface.get().await;
    assert!(matches!(result, Err(Error::Client(_))));

    let node2 = lab.node(2);
    let connected = iface.connect_to_node(&node2.ethernet(0)).await;
    assert!(matches!(connected, Err(Error::Client(_))));

    let network = lab.network(1);
    let connected = iface.connect_to_network(&network).await;
    assert!(matches!(connected, Err(Error::Client(_))));

    node.delete().await?;
    env.teardown().await
}

#[tokio::test]
#[ignore]
async fn interface_connections() -> Result<()> {
    let env = TestEnv::setup().await?;
    let lab = env.client.folder("/")?.lab("test")?;

    let tmpl = env.client.system().node_template("vios").get().await?;
    let node1 = lab.nodes().add(AddNodeRequest::qemu(&tmpl)?).await?;
    let node2 = lab.nodes().add(AddNodeRequest::qemu(&tmpl)?).await?;

    let req = AddNetworkRequest::new("pnet0");
    let network = lab.networks().add(req).await?;

    node1
        .ethernet(0)
        .connect_to_node(&node2.ethernet(0))
        .await?;
    node1.ethernet(1).connect_to_network(&network).await?;

    let topology = lab.topology().await?;
    assert_eq!(topology.len(), 2);
    assert!(
        topology
            .iter()
            .find(|t| t.destination_type == "node"
                && matches!(t.connection_type, InterfaceType::Ethernet))
            .is_some()
    );
    assert!(
        topology
            .iter()
            .find(|t| t.destination_type == "network"
                && matches!(t.connection_type, InterfaceType::Ethernet))
            .is_some()
    );

    node1.ethernet(0).disconnect().await?;
    node1.ethernet(1).disconnect().await?;
    let topology = lab.topology().await?;
    assert_eq!(topology.len(), 0);

    node1.delete().await?;
    node2.delete().await?;
    network.delete().await?;
    env.teardown().await
}

#[tokio::test]
#[ignore]
async fn detach_keeps_the_network() -> Result<()> {
    let env = TestEnv::setup().await?;
    let lab = env.client.folder("/")?.lab("test")?;

    let node1 = env.new_vpcs_node(&lab).await?;
    let node2 = env.new_vpcs_node(&lab).await?;
    node1
        .ethernet(0)
        .connect_to_node(&node2.ethernet(0))
        .await?;
    let bridge = node1.ethernet(0).get().await?.network_id;

    node1.ethernet(0).detach().await?;
    assert!(!node1.ethernet(0).is_connected().await?);
    assert_eq!(node2.ethernet(0).get().await?.network_id, bridge);
    assert!(lab.network(bridge).get().await.is_ok());

    node1.delete().await?;
    node2.delete().await?;
    lab.network(bridge).delete().await?;
    env.teardown().await
}

// TODO: Topology test with open lab
// TODO: Topology test with closed lab
// TODO: Topology test with another opened lab
