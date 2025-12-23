use detox_net::HostAndPort;
use http::Uri;
use paclib::{Engine, Proxy, ProxyOrDirect};

fn find_proxy(cmd: &str, good: &str, bad: &str) {
    let endpoint = "example.org:3128".parse::<HostAndPort>().unwrap();
    let pac_script = format!(
        r#"
        function FindProxyForURL(url, host) {{
            if({cmd} === true) {{
                return "DIRECT";
            }}
            return "PROXY {endpoint}";
        }}
    "#
    );
    let mut eval = Engine::with_pac_script(&pac_script).unwrap();

    assert_eq!(
        ProxyOrDirect::Direct,
        eval.find_proxy(&good.parse::<Uri>().unwrap())
            .unwrap()
            .first()
    );
    assert_eq!(
        ProxyOrDirect::Proxy(Proxy::Http(endpoint)),
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
    find_proxy(
        r#"isResolvable(host)"#,
        "localhost",
        "thishostdoesnotexist.",
    );
}

#[test]
fn test_sh_exp_match() {
    find_proxy(
        r#"shExpMatch(host, "*.example.net")"#,
        "good.example.net",
        "bad.local",
    );

    find_proxy(
        r#"shExpMatch(host, "www?.example.net")"#,
        "www1.example.net",
        "bad.local",
    );
}
#[test]
fn test_my_ip_address() {
    let pac_script = r#"
        function FindProxyForURL(url, host) {{
            myIp = myIpAddress();
            if(myIp.match(/^([a-f0-9:]+:+)+[a-f0-9]+$/) || myIp.match(/^(?:[0-9]{1,3}\.){3}[0-9]{1,3}$/)) {{
                return "DIRECT";
            }}
            return "PROXY example.org:3128";
        }}
    "#;
    let mut eval = Engine::with_pac_script(pac_script).unwrap();

    assert_eq!(
        ProxyOrDirect::Direct,
        eval.find_proxy(&"localhost".parse::<Uri>().unwrap())
            .unwrap()
            .first()
    );
}
