/*
 * @Author: plucky
 * @Date: 2023-11-06 10:30:41
 * @LastEditTime: 2024-4-30 17:10:30
 */

use std::error::Error;
use etcd_client::ConnectOptions;
use etcd_discovery::EtcdTonicDiscovery;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();

    let opt = ConnectOptions::new().with_user("tonic_user", "789789");
    let mut discover = EtcdTonicDiscovery::connect(["127.0.0.1:2379"], Some(opt.to_owned())).await?;
    // 发现前缀为/hello的所有服务
    discover.service_discover("/hello").await?;
    
    // 获取单个节点
    let s = discover.get_service("/hello/1");
    println!("service: {:?}", s);
    // 获取所有节点
    let s = discover.get_all_channel();
    println!("chanel: {:?}", s);

    for _ in 0..5 {
        println!("services: {:?}", discover.get_service_map().read().unwrap());
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
   

    Ok(())
}

#[cfg(test)]
mod tests {
    use etcd_client::*;

    #[tokio::test]
    async fn get_all_keys() {
        let opt = ConnectOptions::new().with_user("root", "r789789");
        let mut client = Client::connect(["127.0.0.1:2379"], Some(opt)).await.unwrap();
        // get all keys
        let resp = client.get("", Some(GetOptions::new().with_all_keys())).await.unwrap();
        for kv in resp.kvs() {
            println!("key: {}, value: {}", kv.key_str().unwrap(), kv.value_str().unwrap());
        }
        
    }

   
}
