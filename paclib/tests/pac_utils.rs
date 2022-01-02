use paclib::{Evaluator, ProxyDesc};

async fn find_proxy(cmd: &str, good: &str, bad: &str) {
    use paclib::Uri;
    let proxy = "http://example.org:3128";
    let pac_script = format!(
        r#"
        function FindProxyForURL(url, host) {{
            if({} === true) {{
                return "DIRECT";
            }}
            return "PROXY {}";
        }}
    "#,
        cmd, proxy
    );
    let proxy = proxy.parse::<Uri>().unwrap();
    let eval = Evaluator::new(&pac_script).unwrap();

    assert_eq!(
        ProxyDesc::Direct,
        eval.find_proxy(good.parse::<Uri>().unwrap())
            .await
            .unwrap()
            .first()
    );
    assert_eq!(
        ProxyDesc::Proxy(proxy),
        eval.find_proxy(bad.parse::<Uri>().unwrap())
            .await
            .unwrap()
            .first()
    );
}

#[tokio::test]
async fn test_is_plain_host_name() {
    find_proxy("isPlainHostName(host)", "www", "example.org").await;
}

#[tokio::test]
async fn test_dns_domain_is() {
    find_proxy(
        r#"dnsDomainIs(host, ".example.org")"#,
        "www.example.org",
        "www",
    )
    .await;
}

#[tokio::test]
async fn test_local_host_or_domain_is() {
    find_proxy(
        r#"localHostOrDomainIs(host, "www.example.org")"#,
        "www",
        "home.example.org",
    )
    .await;
}

#[tokio::test]
async fn test_is_resolvable() {
    find_proxy(r#"isResolvable(host)"#, "localhost", "thishostdoesnotexist").await;
}
