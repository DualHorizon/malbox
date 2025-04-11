use malbox_config::Config;
use malbox_core::communication::common::{ChannelMessage, CommunicationChannel, TaskMessage};
use malbox_core::communication::ipc::host::{self, HostIpc};
use malbox_core::PluginManager;
use malbox_database::{init_database, init_machines};
use malbox_http::http;
use malbox_scheduler::{init_scheduler, ResourceManager, TaskNotificationService};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, subscriber};

mod error;
pub use error::DaemonError;

pub async fn run(config: Config) -> error::Result<()> {
    let db = init_database(&config.database).await;

    let (notification_service, task_receiver) = TaskNotificationService::new();

    // FIXME:
    // init_machines(&db, &config.machinery).await.unwrap();

    // let node = host_ipc.node.as_ref().unwrap();

    // while node.wait(Duration::from_secs(1)).is_ok() {
    //     match host_ipc.receive_message() {
    //         Ok(Some(message)) => match message {
    //             ChannelMessage::Event(event) => {
    //                 debug!("Received event from plugin: {:?}", event.plugin_id);

    //                 debug!("{:#?}", event);
    //             }
    //             _ => debug!("other"),
    //         },
    //         Ok(None) => {}
    //         Err(e) => {
    //             eprintln!("Error receiving message: {:#?}", e);
    //         }
    //     }
    // }

    let resource_manager = Arc::new(ResourceManager::new(db.clone(), config.clone()));

    let mut plugin_manager = PluginManager::new("/home/shard/.config/malbox/plugins/".into());

    plugin_manager.initialize().await.unwrap();

    init_scheduler(
        config.clone(),
        db.clone(),
        resource_manager.clone(),
        task_receiver,
    )
    .await;

    http::serve(config.clone(), db, notification_service)
        .await
        .map_err(|e| DaemonError::Internal(e.to_string()))
}
