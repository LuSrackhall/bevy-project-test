//! Minimal UDP loopback test: dual-stack v6 relay socket ↔ v4 client socket,
//! verifying v4-mapped send_to actually reaches a v4-bound client.

use tokio::net::UdpSocket;

#[tokio::test]
async fn test_udp_loopback_v6relay_v4client() {
    let relay = UdpSocket::bind("[::]:0").await.unwrap();
    let relay_port = relay.local_addr().unwrap().port();
    let client = UdpSocket::bind("0.0.0.0:0").await.unwrap();

    // Client (v4) sends to relay (v6 dual-stack).
    client.send_to(b"hi", format!("127.0.0.1:{}", relay_port)).await.unwrap();

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
