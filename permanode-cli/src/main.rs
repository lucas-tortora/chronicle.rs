use clap::{
    load_yaml,
    App,
    ArgMatches,
};
use futures::SinkExt;
use permanode_broker::application::PermanodeBrokerThrough;
use permanode_common::config::Config;
use scylla::application::ScyllaThrough;
use std::process::Command;
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
};
use url::Url;

#[tokio::main]
async fn main() {
    let yaml = load_yaml!("../cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    if matches.is_present("start") {
        // Assume the permanode exe is in the same location as this one
        let current_exe = std::env::current_exe().unwrap();
        let parent_dir = current_exe.parent().unwrap();
        let permanode_exe = parent_dir.join("permanode.exe");
        if permanode_exe.exists() {
            if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .args(&["/c", "start", "powershell", permanode_exe.to_str().unwrap()])
                    .spawn()
                    .expect("failed to execute process")
            } else {
                Command::new("bash")
                    .arg(permanode_exe.to_str().unwrap())
                    .spawn()
                    .expect("failed to execute process")
            };
        } else {
            panic!("No chronicle exe in the current directory: {}", parent_dir.display());
        }
    } else {
        let config = Config::load().expect("No config file found for Chronicle!");

        if matches.is_present("stop") {
            let (mut stream, _) =
                connect_async(Url::parse(&format!("ws://{}/", config.broker_config.websocket_address)).unwrap())
                    .await
                    .unwrap();
            let message = Message::text(
                serde_json::to_string(&permanode_broker::application::SocketMsg::PermanodeBroker(
                    PermanodeBrokerThrough::ExitProgram,
                ))
                .unwrap(),
            );
            stream.send(message).await.unwrap();
        } else if matches.is_present("rebuild") {
            let (mut stream, _) =
                connect_async(Url::parse(&format!("ws://{}/", config.storage_config.listen_address)).unwrap())
                    .await
                    .unwrap();
            let message = Message::text(
                serde_json::to_string(&scylla::application::SocketMsg::Scylla(ScyllaThrough::Topology(
                    scylla::application::Topology::BuildRing(1),
                )))
                .unwrap(),
            );
            stream.send(message).await.unwrap();
        } else {
            match matches.subcommand() {
                ("nodes", Some(subcommand)) => node(subcommand, config).await,
                ("brokers", Some(subcommand)) => brokers(subcommand, config).await,
                ("archive", Some(subcommand)) => archive(subcommand, config).await,
                _ => (),
            }
        }
    }
}

async fn node<'a>(matches: &ArgMatches<'a>, config: Config) {
    let (mut stream, _) =
        connect_async(Url::parse(&format!("ws://{}/", config.storage_config.listen_address)).unwrap())
            .await
            .unwrap();
    if matches.is_present("list") {
        todo!("Print list of nodes");
    }
    if let Some(address) = matches.value_of("add") {
        let message = scylla::application::SocketMsg::Scylla(ScyllaThrough::Topology(
            scylla::application::Topology::AddNode(address.parse().expect("Invalid address provided!")),
        ));
        let message = Message::text(serde_json::to_string(&message).unwrap());
        stream.send(message).await.unwrap();
    }
    if let Some(address) = matches.value_of("remove") {
        let message = scylla::application::SocketMsg::Scylla(ScyllaThrough::Topology(
            scylla::application::Topology::RemoveNode(address.parse().expect("Invalid address provided!")),
        ));
        let message = Message::text(serde_json::to_string(&message).unwrap());
        stream.send(message).await.unwrap();
    }
}

async fn brokers<'a>(matches: &ArgMatches<'a>, config: Config) {
    match matches.subcommand() {
        ("add", Some(subcommand)) => {
            let mqtt_addresses = subcommand.values_of("mqtt-address").unwrap();
            let endpoint_addresses = subcommand.values_of("endpoint-address");
            // TODO add endpoints
            let (mut stream, _) =
                connect_async(Url::parse(&format!("ws://{}/", config.broker_config.websocket_address)).unwrap())
                    .await
                    .unwrap();
            let mut messages = mqtt_addresses.fold(Vec::new(), |mut list, mqtt_address| {
                list.push(Message::text(
                    serde_json::to_string(&permanode_broker::application::SocketMsg::PermanodeBroker(
                        PermanodeBrokerThrough::Topology(permanode_broker::application::Topology::AddMqttMessages(
                            Url::parse(mqtt_address).unwrap(),
                        )),
                    ))
                    .unwrap(),
                ));
                list.push(Message::text(
                    serde_json::to_string(&permanode_broker::application::SocketMsg::PermanodeBroker(
                        PermanodeBrokerThrough::Topology(
                            permanode_broker::application::Topology::AddMqttMessagesReferenced(
                                Url::parse(mqtt_address).unwrap(),
                            ),
                        ),
                    ))
                    .unwrap(),
                ));
                list
            });
            for message in messages.drain(..) {
                stream.send(message).await.unwrap();
            }
        }
        ("remove", Some(subcommand)) => {
            let id = subcommand.value_of("id").unwrap();
            todo!("Send message");
        }
        ("list", Some(subcommand)) => {
            todo!("List brokers");
        }
        _ => (),
    }
}

async fn archive<'a>(matches: &ArgMatches<'a>, config: Config) {
    match matches.subcommand() {
        ("import", Some(subcommand)) => {
            let dir = subcommand.value_of("directory");
            let range = subcommand.value_of("range");
            todo!("Send message");
        }
        _ => (),
    }
}
