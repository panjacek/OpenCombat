//! Network layer tests (zmq transport).
//! These tests require native libzmq: run them on host docker or any
//! machine with libzmq dev headers installed (see AGENTS.md sandbox bootstrap).

use std::{
    net::TcpListener,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use crossbeam_channel::{unbounded, Receiver};
use zmq as zmq_crate;

use battle_core::{
    config::ChangeConfigMessage,
    message::{
        network::NetworkMessage,
        {InputMessage, Message, OutputMessage},
    },
    network::{client::Client, error::NetworkError, server::Server},
};

const TIMEOUT: Duration = Duration::from_secs(10);

/// Find a free TCP port by binding an ephemeral std listener and releasing it.
/// Tiny race window between release and zmq bind, acceptable for tests.
fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral listener");
    let port = listener.local_addr().expect("local addr").port();
    drop(listener);
    port
}

/// (srv_out_tx, srv_in_rx, cli_in_tx, cli_out_rx, stop_required)
type TestStack = (
    crossbeam_channel::Sender<Vec<OutputMessage>>,
    Receiver<Vec<InputMessage>>,
    crossbeam_channel::Sender<Vec<InputMessage>>,
    Receiver<Vec<OutputMessage>>,
    Arc<AtomicBool>,
);

fn start_server_client() -> TestStack {
    // Server-facing channels
    let (srv_out_tx, srv_out_rx) = unbounded::<Vec<OutputMessage>>();
    let (srv_in_tx, srv_in_rx) = unbounded::<Vec<InputMessage>>();
    // Client-facing channels: req thread drains cli_in_rx, sub thread fills cli_out_tx
    let (cli_in_tx, cli_in_rx) = unbounded::<Vec<InputMessage>>();
    let (cli_out_tx, cli_out_rx) = unbounded::<Vec<OutputMessage>>();

    let stop_required = Arc::new(AtomicBool::new(false));

    let rep_address = format!("tcp://127.0.0.1:{}", free_port());
    let pub_address = format!("tcp://127.0.0.1:{}", free_port());

    let server = Server::new(
        rep_address.clone(),
        pub_address.clone(),
        srv_out_rx,
        srv_in_tx,
        stop_required.clone(),
    );
    server.serve().expect("server serve");

    let sync_required = Arc::new(AtomicBool::new(false));
    let mut client = Client::new(
        rep_address,
        pub_address,
        cli_in_tx.clone(),
        cli_in_rx,
        cli_out_tx,
        cli_out_rx.clone(),
        sync_required,
    );
    client.connect().expect("client connect");

    (srv_out_tx, srv_in_rx, cli_in_tx, cli_out_rx, stop_required)
}

#[test]
fn network_error_from_zmq_error_maps_and_displays() {
    let converted: NetworkError = zmq_crate::Error::EAGAIN.into();
    assert!(matches!(converted, NetworkError::NetworkError(_)));

    let displayed = converted.to_string();
    assert!(
        displayed.starts_with("NetworkError:"),
        "unexpected display: {}",
        displayed
    );

    let direct = NetworkError::SendError("socket gone".to_string());
    assert_eq!(direct.to_string(), "SendError: socket gone");
}

#[test]
fn channel_error_pipe_roundtrips_network_errors() {
    let channel = battle_core::channel::Channel::default();

    channel
        .error_sender()
        .send(NetworkError::ReceiveError("pipe broken".to_string()))
        .expect("send through error pipe");

    let received = channel
        .error_receiver()
        .recv_timeout(TIMEOUT)
        .expect("receive from error pipe");

    assert!(matches!(received, NetworkError::ReceiveError(_)));
}

#[test]
fn message_network_variant_survives_bincode_roundtrip() {
    // The wire contract: Message::Network(Acknowledge) must survive encoding,
    // the server relies on exactly this payload for REP acknowledgements.
    let original = Message::Network(NetworkMessage::Acknowledge);
    let bytes = bincode::serialize(&original).expect("serialize");
    let decoded: Message = bincode::deserialize(&bytes).expect("deserialize");

    assert!(matches!(
        decoded,
        Message::Network(NetworkMessage::Acknowledge)
    ));
}

#[test]
fn req_rep_full_stack_delivers_client_inputs_to_server() {
    let (_srv_out_tx, srv_in_rx, cli_in_tx, _cli_out_rx, stop_required) = start_server_client();

    cli_in_tx
        .send(vec![InputMessage::RequireCompleteSync])
        .expect("queue client input");

    let delivered = srv_in_rx.recv_timeout(TIMEOUT).expect("server got inputs");
    assert_eq!(delivered.len(), 1);
    assert!(matches!(delivered[0], InputMessage::RequireCompleteSync));

    stop_required.store(true, Ordering::Relaxed);
}

#[test]
fn pub_sub_full_stack_delivers_server_outputs_to_client() {
    let (srv_out_tx, _srv_in_rx, _cli_in_tx, cli_out_rx, stop_required) = start_server_client();

    let expected = vec![OutputMessage::ChangeConfig(
        ChangeConfigMessage::TargetCycleDuration(1234),
    )];

    // PUB/SUB has a slow-joiner window: messages published before the
    // subscriber's subscription is active are silently dropped. Retry until
    // the client sees something or we run out of patience.
    let deadline = Instant::now() + TIMEOUT;
    let mut received = None;
    while Instant::now() < deadline {
        srv_out_tx.send(expected.clone()).expect("queue outputs");
        match cli_out_rx.recv_timeout(Duration::from_millis(250)) {
            Ok(messages) => {
                received = Some(messages);
                break;
            }
            Err(_) => continue,
        }
    }

    let messages = received.expect("client never received published outputs");
    assert_eq!(messages.len(), 1);
    assert!(matches!(
        &messages[0],
        OutputMessage::ChangeConfig(ChangeConfigMessage::TargetCycleDuration(1234))
    ));

    stop_required.store(true, Ordering::Relaxed);
}
