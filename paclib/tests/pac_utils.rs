use http::Uri;
use paclib::{Endpoint, Evaluator, ProxyDesc};

fn find_proxy(cmd: &str, good: &str, bad: &str) {
    let endpoint = "example.org:3128".parse::<Endpoint>().unwrap();
    let pac_script = format!(
        r#"
        function FindProxyForURL(url, host) {{
            if({} === true) {{
                return "DIRECT";
            }}
            return "PROXY {}";
        }}
    "#,
        cmd, endpoint
    );
    let mut eval = Evaluator::new(&pac_script).unwrap();

    assert_eq!(
        ProxyDesc::Direct,
        eval.find_proxy(&good.parse::<Uri>().unwrap())
            .unwrap()
            .first()
    );
    assert_eq!(
        ProxyDesc::Proxy(endpoint),
        eval.find_proxy(&bad.parse::<Uri>().unwrap())
            .unwrap()
            .first()
    );
}

#[test]
fn test_is_plain_host_name() {
    find_proxy("isPlainHostName(host)", "www", "example.org");
}

#[test]
fn test_dns_domain_is() {
    find_proxy(
        r#"dnsDomainIs(host, ".example.org")"#,
        "www.example.org",
        "www",
    );
}

#[test]
fn test_local_host_or_domain_is() {
    find_proxy(
        r#"localHostOrDomainIs(host, "www.example.org")"#,
        "www",
        "home.example.org",
    );
}

#[test]
fn test_is_resolvable() {
    find_proxy(r#"isResolvable(host)"#, "localhost", "thishostdoesnotexist");
}
