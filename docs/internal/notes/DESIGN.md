# Design Document

## Edge Cases

- Delete then immediately recreate a manifest (before all of its resources have been cleaned up)
- Delete

## Concepts

### Resource

```rs
struct Resource {
  identity: Identity,
  children: Vec<Identity>,
  depends_on: Vec<Identity>,

  state: Any,
  phase: Any,
}
```

A resource represents *something* on the system, such as a file, a network interface, a container, etc. It contains the following fields:
- `identity`: uniquely identifies the resource across the entire system.
- `children`: a list of identities that references the resources that this one created.
- `depends_on`:
- `spec`: the desired state
- `status`: the actual state
- `phase`: `APPLIED`, `PENDING` (happens when deleting then recreating the resource before it has had the chance of being teardowned), `TEARDOWN`

### Resource Ownership

A resource can own zero or more sub-resources. While not strictly necessary to implement a declarative system, this allows deriving dynamic resources from static configuration files.

For example, the following "master" resource:
```yaml
kind: NetworkInterface
device: eth0
state: up
mtu: 1400
addresses:
  - 10.10.10.2/24
  - 10.10.10.3/24
```

Will create 1 sub-resources + 2 transitive sub-resources:
- `LinkSpec` which will bring up the interface and set the MTU
- The `LinkSpec` itself will have two `AddressSpec` which will assign an address to the Link

### Resource Deletion

When requesting to delete a resource, it will first transition to the `Teardown` phase while being kept in the state.

A resource cannot be deleted if it has any children. it will first delete all it's children, and wait for this process to complete.

Once a resource has no more children, it can run its own teardown logic.

### Config

A config is a user-defined manifests. It behaves much like resources with two major exception:
1. it has no owner and cannot be transitively deleted.
2. 

## Components

### Identity

Objects across the system are uniquely identified by 2 components: the schema and the name.

The schema is a URI to a JSON schema.

### Resource

A resource is something that can be reconciled. It contains the following field:
- `identity` composed
- `owner` which the identity of the owner
- `spec` which contains the desired state
- `status` which contains the computed state

### System Manager

The system manager holds all resources
