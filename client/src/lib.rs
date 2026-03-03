#[cfg(test)]
mod test_connections {
    use std::{env, time::Duration};

    use krill_common::{Blake3BytesRedacted, KrillUtils, OrganizationInfo, ServerOutcome};
    use reqwest::Client;

    #[test]
    fn create_org() {
        let org_details = OrganizationInfo {
            name: "Supervisor".to_string(),
            threshold: 2,
        };

        let bytes = bitcode::encode(&org_details);

        let endpoint = "http://localhost:8080";

        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let client = Client::new();
            let response = client
                .post(endpoint.to_string() + "/create-organization")
                .header("Content-Type", "application/octet-stream")
                .body(
                    jzon::object! {
                        "info": bytes
                    }
                    .to_string(),
                )
                .send()
                .await
                .unwrap()
                .bytes()
                .await
                .unwrap();

            dbg!(&ServerOutcome::<Blake3BytesRedacted>::decode(&response));

            let response = client
                .get(endpoint.to_string() + "/get-organization/" + org_details.name.as_str())
                .header("Content-Type", "application/octet-stream")
                .send()
                .await
                .unwrap()
                .bytes()
                .await
                .unwrap();

            dbg!(&response);

            let decoded = ServerOutcome::<Option<Vec<u8>>>::decode(&response).unwrap();
            match decoded {
                ServerOutcome::Success(value) => {
                    dbg!(bitcode::decode::<OrganizationInfo>(&value.unwrap()));
                }
                ServerOutcome::Failure(error) => {
                    dbg!(error);
                }
            }
        });
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    struct Party {
        name: String,
        port: u16,
    }

    // #[test]
    // fn networked_dkg() {
    //     smol::block_on(async move {
    //         let ports = env::var("TEST_PORTS").expect("Test ports not defined");
    //         let parties_raw = env::var("TEST_PARTIES").expect("Test parties name not defined");

    //         let mut parties = Vec::<Party>::new();
    //         parties_raw.split(",").for_each(|value| {
    //             let name = value.to_string();

    //             parties.push(Party { name, port: 0 });
    //         });

    //         let ports = ports
    //             .split(",")
    //             .map(|value| {
    //                 let port = value.parse::<u16>().expect("Port is not valid u16");
    //                 if port < 1024 {
    //                     panic!("Ports declared may need root privileges");
    //                 }

    //                 port
    //             })
    //             .collect::<Vec<u16>>();

    //         parties.iter_mut().enumerate().for_each(|(index, value)| {
    //             value.port = ports
    //                 .get(index)
    //                 .cloned()
    //                 .expect("Current index indicates not enough ports were passed");
    //         });

    //         parties.dedup();

    //         dbg!(&parties);

    //         let (sender, receiver) = async_channel::unbounded::<String>();

    //         for party in parties.as_slice() {
    //             let party = party.clone();
    //             let sender = sender.clone();

    //             smol::spawn(async move {
    //                 let message = String::from("HEARTBEAT: ")
    //                     + party.name.as_str()
    //                     + "-"
    //                     + party.port.to_string().as_str();
    //                 loop {
    //                     Timer::interval(Duration::from_secs(3)).await;
    //                     sender.send(message.clone()).await.unwrap();
    //                 }
    //             })
    //             .detach();
    //         }

    //         while let Ok(message) = receiver.recv().await {
    //             println!("{message}");
    //         }
    //     });
    // }
}
