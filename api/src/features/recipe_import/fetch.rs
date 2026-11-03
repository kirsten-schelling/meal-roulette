use crate::error::AppError;
use reqwest::{redirect, Url};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

const MAX_BYTES: usize = 2 * 1024 * 1024;
const MAX_REDIRECTS: usize = 5;
const TIMEOUT: Duration = Duration::from_secs(10);

fn blocked() -> AppError {
    AppError::BadRequest("That address isn't allowed.".to_string())
}

fn unreachable_page() -> AppError {
    AppError::Unprocessable("Couldn't load that page.".to_string())
}

fn v4_is_public(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    !(ip.is_unspecified()
        || ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_multicast()
        || ip.is_documentation()
        || a == 0
        // 100.64.0.0/10 shared address space (carrier-grade NAT)
        || (a == 100 && (64..128).contains(&b))
        // 192.0.0.0/24 IETF protocol assignments
        || (a == 192 && b == 0 && c == 0)
        // 198.18.0.0/15 benchmarking
        || (a == 198 && (b == 18 || b == 19))
        // 240.0.0.0/4 reserved
        || a >= 240)
}

fn v6_is_public(ip: Ipv6Addr) -> bool {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return v4_is_public(v4);
    }
    let s = ip.segments();
    // 64:ff9b::/96 (NAT64) and ::/96 (IPv4-compatible) carry an IPv4 address in the tail.
    let embedded = Ipv4Addr::new((s[6] >> 8) as u8, s[6] as u8, (s[7] >> 8) as u8, s[7] as u8);
    if (s[..6] == [0x64, 0xff9b, 0, 0, 0, 0] || s[..6] == [0; 6]) && !v4_is_public(embedded) {
        return false;
    }
    !(ip.is_unspecified()
        || ip.is_loopback()
        || ip.is_multicast()
        // fc00::/7 unique local
        || (s[0] & 0xfe00) == 0xfc00
        // fe80::/10 link local and fec0::/10 site local
        || (s[0] & 0xffc0) == 0xfe80
        || (s[0] & 0xffc0) == 0xfec0
        // 2001:db8::/32 documentation
        || (s[0] == 0x2001 && s[1] == 0x0db8))
}

pub fn ip_is_public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4_is_public(v4),
        IpAddr::V6(v6) => v6_is_public(v6),
    }
}

/// Accepts only an http(s) URL with a host and no embedded credentials.
pub fn parse_url(raw: &str) -> Result<Url, AppError> {
    let not_a_url = || AppError::BadRequest("That doesn't look like a web address.".to_string());
    let url = Url::parse(raw.trim()).map_err(|_| not_a_url())?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(AppError::BadRequest(
            "Only http and https addresses can be imported.".to_string(),
        ));
    }
    if url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() {
        return Err(not_a_url());
    }
    Ok(url)
}

/// Resolves the host and refuses it unless every address it points at passes `allow`. The
/// returned addresses are what the connection must use, so a second DNS answer can't swap in
/// an internal address after this check.
async fn resolve(url: &Url, allow: fn(IpAddr) -> bool) -> Result<Vec<SocketAddr>, AppError> {
    let port = url.port_or_known_default().ok_or_else(blocked)?;
    let no_such_site = || AppError::Unprocessable("Couldn't find that website.".to_string());
    let host = url.host_str().ok_or_else(blocked)?;
    // `host_str()` keeps the brackets on an IPv6 literal ("[::1]"), which `lookup_host` would
    // treat as a DNS name and fail on, so literals are parsed here and never hit DNS.
    let addrs: Vec<SocketAddr> = match host.trim_matches(['[', ']']).parse::<IpAddr>() {
        Ok(ip) => vec![SocketAddr::new(ip, port)],
        Err(_) => tokio::net::lookup_host((host, port))
            .await
            .map_err(|_| no_such_site())?
            .collect(),
    };
    if addrs.is_empty() {
        return Err(no_such_site());
    }
    if addrs.iter().any(|a| !allow(a.ip())) {
        return Err(blocked());
    }
    Ok(addrs)
}

async fn fetch_with(raw: &str, allow: fn(IpAddr) -> bool) -> Result<(Url, String), AppError> {
    let mut url = parse_url(raw)?;
    for _ in 0..=MAX_REDIRECTS {
        let addrs = resolve(&url, allow).await?;
        let client = reqwest::Client::builder()
            .redirect(redirect::Policy::none())
            .timeout(TIMEOUT)
            .connect_timeout(Duration::from_secs(5))
            .user_agent("MealRoulette/0.1 (recipe import)")
            .resolve_to_addrs(url.host_str().ok_or_else(blocked)?, &addrs)
            .build()
            .map_err(|_| unreachable_page())?;
        let mut response = client
            .get(url.clone())
            .header("Accept", "text/html,application/xhtml+xml")
            .send()
            .await
            .map_err(|_| unreachable_page())?;

        let status = response.status();
        if status.is_redirection() {
            let next = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| url.join(v).ok())
                .ok_or_else(unreachable_page)?;
            url = parse_url(next.as_str())?;
            continue;
        }
        if !status.is_success() {
            return Err(AppError::Unprocessable(format!(
                "That page answered with an error ({}).",
                status.as_u16()
            )));
        }
        let is_html = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| {
                let v = v.to_ascii_lowercase();
                v.contains("text/html") || v.contains("application/xhtml")
            });
        if !is_html {
            return Err(AppError::Unprocessable(
                "That address isn't a web page.".to_string(),
            ));
        }
        let too_big = || AppError::Unprocessable("That page is too large to import.".to_string());
        if response
            .content_length()
            .is_some_and(|n| n > MAX_BYTES as u64)
        {
            return Err(too_big());
        }
        let mut body: Vec<u8> = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| unreachable_page())? {
            if body.len() + chunk.len() > MAX_BYTES {
                return Err(too_big());
            }
            body.extend_from_slice(&chunk);
        }
        return Ok((url, String::from_utf8_lossy(&body).into_owned()));
    }
    Err(AppError::Unprocessable(
        "That address redirects too many times.".to_string(),
    ))
}

/// Fetches a web page for import: http(s) only, public addresses only (checked again on every
/// redirect), a time limit and a size cap. Returns the final URL and the page's HTML.
pub async fn fetch_html(raw: &str) -> Result<(Url, String), AppError> {
    fetch_with(raw, ip_is_public).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn private_and_special_addresses_are_refused() {
        for s in [
            "127.0.0.1",
            "10.1.2.3",
            "172.16.0.1",
            "192.168.1.1",
            "169.254.169.254",
            "0.0.0.0",
            "100.64.0.1",
            "224.0.0.1",
            "255.255.255.255",
            "::1",
            "::",
            "fe80::1",
            "fc00::1",
            "fd12:3456::1",
            "::ffff:127.0.0.1",
            "::ffff:10.0.0.1",
            "64:ff9b::7f00:1",
            "2001:db8::1",
        ] {
            assert!(!ip_is_public(ip(s)), "{s} should be refused");
        }
    }

    #[test]
    fn public_addresses_are_allowed() {
        for s in ["93.184.216.34", "8.8.8.8", "2606:4700:4700::1111"] {
            assert!(ip_is_public(ip(s)), "{s} should be allowed");
        }
    }

    #[test]
    fn only_http_and_https_urls_are_accepted() {
        assert!(parse_url("https://example.com/r").is_ok());
        assert!(parse_url(" http://example.com ").is_ok());
        for bad in [
            "file:///etc/passwd",
            "ftp://example.com/x",
            "javascript:alert(1)",
            "example.com",
            "https://user:pw@example.com/",
            "",
        ] {
            assert!(parse_url(bad).is_err(), "{bad} should be refused");
        }
    }

    #[tokio::test]
    async fn loopback_hosts_are_refused_before_connecting() {
        for url in [
            "http://127.0.0.1:1/",
            "http://localhost:1/",
            "http://[::1]:1/",
        ] {
            let err = fetch_html(url).await.unwrap_err();
            assert!(matches!(err, AppError::BadRequest(_)), "{url}: {err}");
        }
    }

    async fn serve(response: String) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 2048];
            let _ = socket.read(&mut buf).await;
            let _ = socket.write_all(response.as_bytes()).await;
        });
        port
    }

    fn page(content_type: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    fn allow_all(_: IpAddr) -> bool {
        true
    }

    #[tokio::test]
    async fn fetches_a_page() {
        let port = serve(page("text/html; charset=utf-8", "<p>hi</p>")).await;
        let (url, html) = fetch_with(&format!("http://127.0.0.1:{port}/a"), allow_all)
            .await
            .unwrap();
        assert_eq!(html, "<p>hi</p>");
        assert_eq!(url.path(), "/a");
    }

    #[tokio::test]
    async fn oversized_pages_are_refused() {
        let port = serve(page("text/html", &"x".repeat(MAX_BYTES + 1))).await;
        let err = fetch_with(&format!("http://127.0.0.1:{port}/"), allow_all)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("too large"), "{err}");
    }

    #[tokio::test]
    async fn non_html_responses_are_refused() {
        let port = serve(page("application/json", "{}")).await;
        assert!(fetch_with(&format!("http://127.0.0.1:{port}/"), allow_all)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn redirects_are_checked_against_the_address_rule() {
        fn only_127_0_0_1(ip: IpAddr) -> bool {
            ip == IpAddr::V4(Ipv4Addr::LOCALHOST)
        }
        let port = serve(
            "HTTP/1.1 302 Found\r\nLocation: http://127.0.0.2:9/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                .to_string(),
        )
        .await;
        let err = fetch_with(&format!("http://127.0.0.1:{port}/"), only_127_0_0_1)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)), "{err}");
    }
}
