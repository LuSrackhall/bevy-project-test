//! 双栈 UDP 回归测试：v6 双栈 relay socket ↔ v4 客户端，验证 v4-mapped 可达。
//!
//! 这条用例同时守住一个真实缺陷：`IPV6_V6ONLY` 的默认值在 Windows 上是 1
//! （Linux/macOS 为 0），因此朴素的 `bind("[::]")` 在 Windows 上只收 IPv6，
//! IPv4 客户端永远连不上。生产代码改走 `transport::bind_dual_stack_udp`
//! （显式 `set_only_v6(false)`），本用例即其跨平台回归防线。
//!
//! macOS 上默认跳过：双栈接收**必须**绑 v6 通配地址，而这会触发系统防火墙的
//! 入站授权弹窗。CI 的 Linux 与 Windows 覆盖此用例；需要本机验证时用
//! `cargo test -p bevy_adapter --test udp_loopback -- --ignored`。

use bevy_adapter::transport::bind_dual_stack_udp;
use tokio::net::UdpSocket;

#[tokio::test]
#[cfg_attr(
    target_os = "macos",
    ignore = "双栈接收必须绑 v6 通配地址，会触发 macOS 防火墙弹窗；由 CI 的 Linux/Windows 覆盖"
)]
async fn test_udp_loopback_v6relay_v4client() {
    let relay = bind_dual_stack_udp(0).unwrap();
    let relay_port = relay.local_addr().unwrap().port();
    let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    // Client (v4) sends to relay (v6 dual-stack).
    client
        .send_to(b"hi", format!("127.0.0.1:{}", relay_port))
        .await
        .unwrap();

    // Relay receives (source appears v4-mapped on a dual-stack socket).
    let mut buf = [0u8; 64];
    let (n, from) = relay.recv_from(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], b"hi");

    // Relay sends back to the (possibly v4-mapped) source address.
    let res = relay.send_to(b"reply", from).await;
    assert!(res.is_ok(), "v4-mapped send_to must succeed, got {:?}", res);

    // Client (v4) must receive the reply.
    let mut buf2 = [0u8; 64];
    let (n2, _) = client.recv_from(&mut buf2).await.unwrap();
    assert_eq!(&buf2[..n2], b"reply");
}
