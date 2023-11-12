/*
 * @Author: anger
 * @Date: 2023-11-10 15:42:55
 * @LastEditTime: 2023-11-12 09:26:55
 */


mod etcd_discovery;
mod etcd_register;
mod etcd_discovery_base;

pub use etcd_discovery::*;
pub use etcd_register::*;
pub use etcd_discovery_base::*;

// #[doc(hidden)]
/// Re-export etcd_client
#[doc(inline)]
pub use etcd_client;

pub use etcd_client::ConnectOptions;
