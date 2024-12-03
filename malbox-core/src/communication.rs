use super::{
    manager::state::PluginState,
    plugin::PluginRequirements,
    types::{PluginEvent, PluginType},
};
use iceoryx2::port::{
    listener::Listener, notifier::Notifier, publisher::Publisher, subscriber::Subscriber,
};
use iceoryx2::prelude::*;
use std::{collections::HashMap, time::Duration};

#[derive(Debug, Clone)]
pub struct PluginMessage {
    pub plugin_id: String,
    pub data: Vec<u8>,
}

pub struct PluginCommunication {
    publishers: HashMap<PluginType, Publisher<ipc::Service, PluginMessage, ()>>,
    subscribers: HashMap<PluginType, Subscriber<ipc::Service, PluginMessage, ()>>,
    event_notifier: Notifier<ipc::Service>,
    event_listener: Listener<ipc::Service>,
}

impl PluginCommunication {
    pub fn new(node: &Node<ipc::Service>) -> Result<Self, anyhow::Error> {
        let mut publishers = HashMap::new();
        let mut subscribers = HashMap::new();

        for plugin_type in [
            PluginType::Storage,
            PluginType::Network,
            PluginType::Scheduler,
            PluginType::Analysis,
            PluginType::Monitor,
        ]
        .iter()
        {
            let service_name = ServiceName::new(&format!("plugin.{:?}", plugin_type))?;
            let service = node
                .service_builder(&service_name)
                .publish_subscribe::<PluginMessage>()
                .open_or_create()?;

            publishers.insert(plugin_type.clone(), service.publisher_builder().create()?);
            subscribers.insert(plugin_type.clone(), service.subscriber_builder().create()?);
        }

        let event_service = node
            .service_builder(&"plugin.events".try_into()?)
            .event()
            .open_or_create()?;

        Ok(Self {
            publishers,
            subscribers,
            event_notifier: event_service.notifier_builder().create()?,
            event_listener: event_service.listener_builder().create()?,
        })
    }

    pub fn send_start_signal(&self, plugin: &PluginRequirements) -> Result<(), anyhow::Error> {
        if let Some(publisher) = self.publishers.get(&plugin.plugin_type) {
            let message = PluginMessage {
                plugin_id: "start".into(),
                data: vec![1],
            };
            let sample = publisher.loan_uninit()?;
            let sample = sample.write_payload(message);
            sample.send()?;
        }
        Ok(())
    }

    pub fn process_messages(&mut self, state: &mut PluginState) -> Result<(), anyhow::Error> {
        for (_, subscriber) in &self.subscribers {
            while let Some(_data) = subscriber.receive()? {
                todo!()
            }
        }

        while let Ok(Some(event_id)) = self.event_listener.timed_wait_one(Duration::from_millis(0))
        {
            let event = match event_id.as_value() {
                1 => Some(PluginEvent::Started("plugin-id".to_string())),
                2 => Some(PluginEvent::Completed("plugin-id".to_string())),
                _ => None,
            };

            if let Some(event) = event {
                state.handle_event(event);
            }
        }

        Ok(())
    }
}
