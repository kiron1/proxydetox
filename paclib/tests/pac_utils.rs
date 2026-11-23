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
fn test_weekday_range_reversed_bounds() {
    let pac_script = r#"
        function FindProxyForURL(url, host) {
            var weekdays = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
            var current = new Date().getDay();
            for (var first = 1; first < weekdays.length; first++) {
                for (var second = 0; second < first; second++) {
                    var expected = current == first || current == second;
                    if (weekdayRange(weekdays[first], weekdays[second]) != expected) {
                        return "PROXY example.org:3128";
                    }
                }
            }
            return "DIRECT";
        }
    "#;
    let mut eval = Engine::with_pac_script(pac_script).unwrap();

    assert_eq!(
        ProxyOrDirect::Direct,
        eval.find_proxy(&"http://localhost/".parse::<Uri>().unwrap())
            .unwrap()
            .first()
    );
}

#[test]
fn test_time_range_reversed_bounds() {
    let pac_script = r#"
        function FindProxyForURL(url, host) {
            var current = new Date().getHours();
            var first = current == 23 ? 23 : current + 1;
            var second = current == 23 ? 22 : current;
            var expected = current == first || current == second;
            return timeRange(first, second) == expected
                ? "DIRECT"
                : "PROXY example.org:3128";
        }
    "#;
    let mut eval = Engine::with_pac_script(pac_script).unwrap();

    assert_eq!(
        ProxyOrDirect::Direct,
        eval.find_proxy(&"http://localhost/".parse::<Uri>().unwrap())
            .unwrap()
            .first()
    );
}

#[test]
fn test_date_range_exact_day_and_month() {
    let pac_script = r#"
        function FindProxyForURL(url, host) {
            var months = ["JAN", "FEB", "MAR", "APR", "MAY", "JUN",
                "JUL", "AUG", "SEP", "OCT", "NOV", "DEC"];
            var current = new Date();
            for (var month = 0; month < months.length; month++) {
                for (var day = 1; day <= 31; day++) {
                    var expected = current.getMonth() == month &&
                        current.getDate() == day;
                    if (dateRange(day, months[month]) != expected) {
                        return "PROXY example.org:3128";
                    }
                }
            }
            return "DIRECT";
        }
    "#;
    let mut eval = Engine::with_pac_script(pac_script).unwrap();

    assert_eq!(
        ProxyOrDirect::Direct,
        eval.find_proxy(&"http://localhost/".parse::<Uri>().unwrap())
            .unwrap()
            .first()
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
