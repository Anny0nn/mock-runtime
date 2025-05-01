use serde_json::Value;
use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
    ext::subxt_rpcs::client::RpcParams,
    utils::H256,
    SubstrateConfig,
};

pub async fn get_client() -> RpcClient {
    let client = RpcClient::from_url("ws://localhost:9944")
        .await
        .expect("Node should be running");
    client
}

pub async fn get_legacy_rpc_methods(client: &RpcClient) -> LegacyRpcMethods<SubstrateConfig> {
    LegacyRpcMethods::new(client.clone())
}

pub async fn get_block_hash(client: &RpcClient) -> H256 {
    client
        .request("chain_getBlockHash", RpcParams::default())
        .await
        .unwrap()
}

pub async fn get_rpc_methods(client: &RpcClient) -> Vec<String> {
    let methods_map: std::collections::HashMap<String, Vec<String>> = client
        .request("rpc_methods", RpcParams::default())
        .await
        .unwrap();
    methods_map.get("methods").unwrap().to_owned()
}

pub async fn get_block_number_as_json_value(client: &RpcClient) -> Value {
    client
        .request("chain_getBlock", RpcParams::default())
        .await
        .unwrap()
}

#[tokio::main]
async fn main() {
    let client = get_client().await;
    let block_hash = get_block_hash(&client).await;
    println!("Block hash: {:?}", block_hash);
    println!("\n");
    let rpc_methods = get_rpc_methods(&client).await;
    println!("RPC methods: {:?}", rpc_methods);
    println!("\n");
    let block_number = get_block_number_as_json_value(&client).await;
    println!("Block number: {:?}", block_number);
    println!("\n");
}
