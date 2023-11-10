# Etcd discovery and register for tonic

<div align="center">
  <!-- Version -->
  <a href="https://crates.io/crates/etcd-discovery">
    <img src="https://img.shields.io/crates/v/etcd-discovery.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  
  <!-- Docs -->
  <a href="https://docs.rs/etcd-discovery">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/etcd-discovery">
    <img src="https://img.shields.io/crates/d/etcd-discovery.svg?style=flat-square"
      alt="Download" />
  </a>
</div>

## Use
``` toml
[dependencies]
tokio = { version = "1", features = ["macros","rt-multi-thread"] }
tonic = "0.10"
etcd-discovery = "0.1"
```

# server

``` rust
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051";
    
    // 使用etcd注册服务
    let opt = ConnectOptions::new().with_user("tonic_user", "789789");
    let mut register = EtcdRegister::connect(["127.0.0.1:2379"], Some(opt)).await?;
    register.lease_grant(30, 10).await?;
    register.put("/hello/1", format!("http://{addr}")).await?;
    
    let greeter = MyGreeter::default();
    let svc = GreeterServer::new(greeter);

    tracing::info!("GreeterServer listening on: {}", addr);
    
    Server::builder()
        // 使用拦截器
        // .layer(tonic::service::interceptor(check_auth))
        .add_service(svc)
        .serve(addr.parse()?)
        .await?;

    Ok(())
}
```

# client

``` rust

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();
    // 使用etcd服务发现
    let opt = ConnectOptions::new().with_user("tonic_user", "789789");
    let mut discover = EtcdDiscovery::connect(["127.0.0.1:2379"], Some(opt)).await?;
    discover.service_discover("/hello").await?;
    
    let channel = discover.get_service("/hello/1").unwrap();
    let mut client = GreeterClient::new(channel);

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;
    println!("RESPONSE={:?}", response);

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
   

    Ok(())
}

``````