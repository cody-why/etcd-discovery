/*
 * @Author: plucky
 * @Date: 2023-11-06 10:30:41
 * @LastEditTime: 2023-11-10 20:18:30
 */

use std::error::Error;

use etcd_client::ConnectOptions;
use etcd_discovery::{EtcdTonicDiscovery, EtcdRegister};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    // 服务发现
    let opt = ConnectOptions::new().with_user("tonic_user", "789789");
    let mut discover = EtcdTonicDiscovery::connect(["127.0.0.1:2379"], Some(opt.to_owned())).await?;
    discover.service_discover("/hello").await?;
    
    tokio::spawn(async move {
        // 服务注册
        let mut registry = EtcdRegister::connect(["127.0.0.1:2379"], Some(opt)).await.unwrap();
        registry.lease_grant(30, 10).await.unwrap();
        
        registry.put("/hello/1", "http://world").await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        registry.put("/hello/2", "http://world.or").await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        registry.delete("/hello/1").await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    });

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    let s = discover.get_service("/hello/1");
    println!("service: {:?}", s);
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
