use crate::types::{PluginEvent, PluginType};

use super::PluginMessage;

use iceoryx2::port::{
    event_id::EventId, notifier::Notifier, publisher::Publisher, subscriber::Subscriber,
};
use iceoryx2::prelude::*;
use iceoryx2::sample::Sample;

// For plugin processes/programs
pub struct PluginCommunication {
    pub data_subscriber: Subscriber<ipc::Service, PluginMessage, ()>,
    pub result_publisher: Publisher<ipc::Service, PluginMessage, ()>,
    pub event_notifier: Notifier<ipc::Service>,
    pub plugin_type: PluginType,
    pub plugin_id: String, // NOTE: TBD
}

impl PluginCommunication {
    pub fn new(
        node: &Node<ipc::Service>,
        plugin_type: PluginType,
        id: String,
    ) -> anyhow::Result<Self> {
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
            data_subscriber: data_service.subscriber_builder().create()?,
            result_publisher: result_service.publisher_builder().create()?,
            event_notifier: event_service.notifier_builder().create()?,
            plugin_type,
            plugin_id: id,
        })
    }

    pub fn receive_data(&self) -> anyhow::Result<Option<Sample<ipc::Service, PluginMessage, ()>>> {
        if let Some(sample) = self.data_subscriber.receive()? {
            // NOTE: we should add additional conditions that are optional
            // for example, checks for specific plugin ID, not just type.
            if sample.payload().plugin_type == self.plugin_type {
                return Ok(Some(sample));
            }
        }
        Ok(None)
    }

    pub fn send_result(&self, data: Vec<u8>) -> anyhow::Result<()> {
        let message = PluginMessage {
            plugin_id: self.plugin_id.clone(),
            plugin_type: self.plugin_type.clone(),
            data,
        };
        let sample = self.result_publisher.loan_uninit()?;
        let sample = sample.write_payload(message);
        sample.send()?;
        Ok(())
    }

    pub fn notify_event(&self, event: PluginEvent) -> anyhow::Result<()> {
        let event_id = match &event {
            PluginEvent::ResourceReady { .. } => EventId::new(1),
            PluginEvent::Started(_) => EventId::new(2),
            PluginEvent::Failed(_, _) => EventId::new(3),
            PluginEvent::Shutdown(_) => EventId::new(4),
        };

        self.event_notifier.notify_with_custom_event_id(event_id)?;
        Ok(())
    }
}
