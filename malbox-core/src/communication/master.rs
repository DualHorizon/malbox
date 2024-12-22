use crate::types::{PluginEvent, PluginType};

use super::PluginMessage;

use iceoryx2::port::{listener::Listener, publisher::Publisher, subscriber::Subscriber};
use iceoryx2::prelude::*;
use iceoryx2::sample::Sample;
use std::time::Duration;

// Our main/master process/program
pub struct MasterCommunication {
    pub data_publisher: Publisher<ipc::Service, PluginMessage, ()>,
    pub result_subscriber: Subscriber<ipc::Service, PluginMessage, ()>,
    pub event_listener: Listener<ipc::Service>,
}

impl MasterCommunication {
    pub fn new(node: &Node<ipc::Service>) -> anyhow::Result<Self> {
        let data_service = node
            .service_builder(&"plugin.data".try_into()?)
            .publish_subscribe::<PluginMessage>()
            .open_or_create()?;

        let result_service = node
            .service_builder(&"plugin.results".try_into()?)
            .publish_subscribe::<PluginMessage>()
            .open_or_create()?;

        let event_service = node
            .service_builder(&"plugin.events".try_into()?)
            .event()
            .open_or_create()?;

        Ok(Self {
            data_publisher: data_service.publisher_builder().create()?,
            result_subscriber: result_service.subscriber_builder().create()?,
            event_listener: event_service.listener_builder().create()?,
        })
    }

    pub fn send_data(&self, plugin_type: PluginType, data: Vec<u8>) -> anyhow::Result<()> {
        let message = PluginMessage {
            plugin_id: String::new(), // NOTE: TBD
            plugin_type,
            data,
        };
        let sample = self.data_publisher.loan_uninit()?;
        let sample = sample.write_payload(message);
        sample.send()?;
        Ok(())
    }

    pub fn receive_result(
        &self,
    ) -> anyhow::Result<Option<Sample<ipc::Service, PluginMessage, ()>>> {
        Ok(self.result_subscriber.receive()?)
    }

    pub fn receive_events(&self) -> anyhow::Result<Option<PluginEvent>> {
        if let Ok(Some(event_id)) = self.event_listener.timed_wait_one(Duration::from_millis(0)) {
            let event = match event_id.as_value() {
                1 => Some(PluginEvent::ResourceReady {
                    id: "plugin-id".to_string(),
                    plugin_type: PluginType::Analysis,
                }),
                2 => Some(PluginEvent::Started("plugin-id".to_string())),
                3 => Some(PluginEvent::Failed(
                    "plugin-id".to_string(),
                    "error".to_string(),
                )),
                4 => Some(PluginEvent::Shutdown("plugin-id".to_string())),
                _ => None,
            };
            return Ok(event);
        }
        Ok(None)
    }
}
