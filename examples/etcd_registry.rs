/*
 * @Author: plucky
 * @Date: 2023-11-06 10:30:41
 * @LastEditTime:  2024-4-30 17:10:30
 */

use std::error::Error;
use etcd_client::ConnectOptions;
use etcd_discovery::EtcdRegister;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
   
    let opt = ConnectOptions::new().with_user("tonic_user", "789789");
    
    let mut registry = EtcdRegister::connect(["127.0.0.1:2379"], Some(opt)).await.unwrap();
    registry.lease_grant(30, 10).await.unwrap();
    
    registry.put("/hello/1", "http://world").await.unwrap(); // 服务注册

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    registry.put("/hello/2", "http://world.or").await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    registry.delete("/hello/1").await.unwrap(); // 服务注销


    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    
    

    Ok(())
}
