#import "../lib.typ": *

= Exemple de configuration complète <appendix-full-config>
```yaml
---
schema: api
auth: "none"

---
schema: network:dns
nameservers:
  - 9.9.9.9

---
schema: network:link
name: eth0
admin_up: true

---
schema: network:dhcp
name: eth0

---
schema: container:runtime
name: rootfull
engine: podman
uid: 0
gid: 0
depends_on:
  - network:dns
  - network:route/eth0-dhcp

---
schema: container:instance
name: http
image: docker.io/fredericarr/simple-http-server:latest
runtime: rootfull
ports:
  - container_port: 80
    host_port: 8080

```
