use crate::server_tests::common::quic::QuicClientFactory;
use crate::server_tests::common::scenarios::{
    consumer_group_join_scenario, consumer_group_with_multiple_clients_polling_messages_scenario,
    consumer_group_with_single_client_polling_messages_scenario, message_headers_scenario,
    system_scenario,
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn system_and_consumer_group_scenarios_should_be_valid() {
    let client_factory = QuicClientFactory {};
    system_scenario::run(&client_factory).await;
    message_headers_scenario::run(&client_factory).await;
    consumer_group_join_scenario::run(&client_factory).await;
    consumer_group_with_single_client_polling_messages_scenario::run(&client_factory).await;
    consumer_group_with_multiple_clients_polling_messages_scenario::run(&client_factory).await;
}
