/*
 * @Author: plucky
 * @Date: 2023-11-06 15:04:59
 * @LastEditTime: 2024-4-29 09:31:22
 */

use std::{collections::HashMap, sync::{RwLock, Arc}, time::Duration, str::FromStr};

use etcd_client::*;
use tokio::sync::mpsc::Sender;
use tonic::transport::{Channel, Endpoint};
use tracing::info;
use tower::discover::Change;

/// etcd 服务发现 for tonic
#[allow(dead_code)]
pub struct EtcdTonicDiscovery {
    etcd_client: Client,
    service_map: Arc<RwLock<HashMap<String, Channel>>>,
    // 所有前缀服务的channel
    tonic_channel: Channel,
    // 通知all_channel服务变化
    rx: Sender<Change<String, Endpoint>>,
}

impl EtcdTonicDiscovery {
    /// 获取所有前缀服务的channel, 负载均衡的channel
    pub fn get_all_channel(&self) -> Channel {
        self.tonic_channel.clone()
    }

    /// 获取所有前缀服务的map
    pub fn get_service_map(&self) -> Arc<RwLock<HashMap<String, Channel>>> {
        self.service_map.clone()
    }

    /// 获取etcd客户端
    pub fn get_etcd_client(&self) -> Client {
        self.etcd_client.clone()
    }
}

impl EtcdTonicDiscovery {
    /// 用etcd_client创建服务发现
    pub fn new(client: Client) -> Self {
        let (channel, rx) = Channel::balance_channel(256);
        
        Self {
            etcd_client: client,
            service_map: Arc::new(RwLock::new(HashMap::new())),
            tonic_channel: channel,
            rx,
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
        let rx = self.rx.clone();

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
                                Self::add_service_map(&rx,&service_map, key, value).await;
                            }
                            
                        }
                        etcd_client::EventType::Delete => {
                            if let Some(kv) = event.kv(){
                                let key = kv.key_str().unwrap_or_default();
                                info!("discover watch delete key: {}", key);
                                Self::remove_service_map(&rx,&service_map, key).await;
                            }
                        }
                    }
                }
            }
            watcher.cancel().await.unwrap();

        });


        Ok(())
    }

    /// 获取一个服务的channel, 例如: /hello/1
    pub fn get_service(&self, key: impl AsRef<str>) -> Option<Channel> {
        self.service_map.read().unwrap().get(key.as_ref()).cloned()
    }
    /// 删除一个服务
    pub fn remove_service(&mut self, key: impl AsRef<str>) -> Option<Channel> {
        self.service_map.write().unwrap().remove(key.as_ref())
    }
    /// 手动添加一个服务
    pub async fn add_service(&self, key: impl AsRef<str>, url: &str) {
        Self::add_service_map(&self.rx, &self.service_map, key.as_ref(), url).await;
        
    }

    #[inline]
    async fn new_endpoint(uri: &str, timeout: u64) -> Result<Endpoint, tonic::transport::Error>{
        Ok(Endpoint::from_str(uri)?
            .timeout(Duration::from_secs(timeout))
            )
        
    }

    #[inline]
    async fn add_service_map(rx: &Sender<Change<String, Endpoint>>, service_map: &RwLock<HashMap<String, Channel>>, key: impl Into<String>, value: &str) {
        let key = key.into();
        if let Ok(channel) = Self::new_endpoint(value, 10).await {
            service_map
                .write()
                .unwrap()
                .insert(key.clone(), channel.connect_lazy());
            rx.try_send(Change::Insert(key, channel)).unwrap();

        }else {
            tracing::info!("tonic endpoint connect error: {:?}", value);
        }
        
    }
    
    #[inline]
    async fn remove_service_map(rx: &Sender<Change<String, Endpoint>>, service_map: &RwLock<HashMap<String, Channel>>, key: impl AsRef<str>) {
        service_map.write().unwrap().remove(key.as_ref());
        rx.try_send(Change::Remove(key.as_ref().into())).unwrap();
    }

}



#[cfg(test)]
mod tests {
    use etcd_client::*;
    use crate::etcd_tonic_discovery::EtcdTonicDiscovery;

    #[tokio::test]
    async fn test_discovery() -> Result<(), etcd_client::Error> {
        tracing_subscriber::fmt().init();
        let opt = ConnectOptions::new().with_user("root", "789789");
        let mut discover = EtcdTonicDiscovery::connect(["127.0.0.1:2379"], Some(opt)).await?;
        discover.service_discover("/hello/1").await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        Ok(())
    
    }

}