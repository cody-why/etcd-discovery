/*
 * @Author: plucky
 * @Date: 2023-11-06 15:04:59
 * @LastEditTime: 2023-11-12 09:32:04
 */

use std::{collections::HashMap, sync::{RwLock, Arc}};

use etcd_client::*;
use tracing::info;

/// etcd discovery base, get service addr String
#[allow(dead_code)]
pub struct EtcdDiscoveryBase {
    etcd_client: Client,
    service_map: Arc<RwLock<HashMap<String, String>>>,
    
}

impl EtcdDiscoveryBase {

    /// 获取所有服务的map
    pub fn get_service_map(&self) -> Arc<RwLock<HashMap<String, String>>> {
        self.service_map.clone()
    }

    /// 获取etcd客户端
    pub fn get_etcd_client(&self) -> Client {
        self.etcd_client.clone()
    }
}

impl EtcdDiscoveryBase {
    /// 用etcd_client创建服务发现
    pub fn new(client: Client) -> Self {
        
        Self {
            etcd_client: client,
            service_map: Arc::new(RwLock::new(HashMap::new())),
            
        }
    }

    /// 连接etcd, 创建服务发现
    pub async fn connect(etcd_addr: impl AsRef<[&str]>, options: Option<ConnectOptions>) -> Result<Self, Error> {
        let client = Client::connect(etcd_addr, options).await?;
        info!("etcd connect success");
        Ok(Self::new(client))
    }
    
    /// 服务发现, prefix为服务前缀, 例如: /hello, 发现前缀/hello的所有服务
    pub async fn service_discover(&mut self, prefix: &str) -> Result<(), Error> {
        let opt = Some(GetOptions::new().with_prefix());
        let resp = self.etcd_client.get(prefix, opt).await?;
        for kv in resp.kvs() {
            let key = kv.key_str().unwrap_or_default();
            let value = kv.value_str().unwrap_or_default();
            info!("discover put key: {} value: {}", key, value);
            self.add_service(key, value).await;
        }

        let opt = Some(WatchOptions::new().with_prefix());
        let (mut watcher, mut stream) = self.etcd_client.watch(prefix, opt).await?;
        let service_map = self.service_map.clone();

        tokio::spawn(async move {
            while let Some(resp) = stream.message().await.unwrap() {
                for event in resp.events() {
                    match event.event_type() {
                        etcd_client::EventType::Put => {
                            if let Some(kv) = event.kv(){
                                let key = kv.key_str().unwrap_or_default();
                                let value = kv.value_str().unwrap_or_default();
                                info!("discover watch put key: {} value: {}", key, value);
                                if key.is_empty() {
                                    continue
                                }
                                Self::add_service_map(&service_map, key, value).await;
                            }
                            
                        }
                        etcd_client::EventType::Delete => {
                            if let Some(kv) = event.kv(){
                                let key = kv.key_str().unwrap_or_default();
                                info!("discover watch delete key: {}", key);
                                Self::remove_service_map(&service_map, key).await;
                            }
                        }
                    }
                }
            }
            watcher.cancel().await.unwrap();

        });


        Ok(())
    }

    /// 获取一个服务的地址, 例如: /hello/1
    pub fn get_service(&self, key: impl AsRef<str>) -> Option<String> {
        self.service_map.read().unwrap().get(key.as_ref()).cloned()
    }
    
    pub fn remove_service(&mut self, key: impl AsRef<str>) -> Option<String> {
        self.service_map.write().unwrap().remove(key.as_ref())
    }

    pub async fn add_service(&self, key: impl AsRef<str>, url: &str) {
        Self::add_service_map(&self.service_map, key.as_ref(), url).await;
        
    }


    #[inline]
    async fn add_service_map( service_map: &RwLock<HashMap<String, String>>, key: impl Into<String>, value: &str) {
        let key = key.into();
        service_map
                .write()
                .unwrap()
                .insert(key.clone(), value.into());
           
        
    }
    #[inline]
    async fn remove_service_map( service_map: &RwLock<HashMap<String, String>>, key: impl AsRef<str>) {
        service_map.write().unwrap().remove(key.as_ref());
    }

}



#[cfg(test)]
mod tests {
    use etcd_client::*;
    use super::*;

    #[tokio::test]
    async fn test_discovery() -> Result<(), etcd_client::Error> {
        tracing_subscriber::fmt().init();
        let opt = ConnectOptions::new().with_user("root", "789789");
        let mut discover = EtcdDiscoveryBase::connect(["127.0.0.1:2379"], Some(opt)).await?;
        discover.service_discover("/hello/1").await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        Ok(())
    
    }

}