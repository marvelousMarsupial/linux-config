# Burp Setup

Burp Suite proxy configuration for web testing.

## Start Burp

```bash
burpsuite
# Or with more memory
java -Xmx4g -jar /opt/burpsuite/burpsuite.jar
```

## Proxy Config

1. Burp: Proxy → Options → 127.0.0.1:8080
2. Firefox: Settings → Network → Manual proxy → 127.0.0.1:8080
3. Import CA: http://burp/cert (for HTTPS)

## FoxyProxy (optional)

Firefox extension to toggle proxy on/off quickly.

## Scope

Target → Scope → Add target URL to limit what Burp captures.

See also: [[attack-box]]
