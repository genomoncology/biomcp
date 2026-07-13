# DisGeNET Credential Rejection

DisGeNET sections require provider credentials, and scripts need to distinguish a
missing local key from a configured key that the provider will not accept. This
routine contract uses local HTTP transport so no live account enters the gate.

## Configured DisGeNET credentials rejected by the provider

When DisGeNET rejects a configured credential, scripts receive a distinct error
from a missing local key. The message covers both an invalid credential and an
account whose plan does not include API access, without printing the credential.

<!-- mustmatch-lint: skip -->

```bash run id=disgenet-rejected-key exit=1
biomcp --json get gene BRAF disgenet
```

```json expect=disgenet-rejected-key contains
{
  "error": {
    "code": "api_key_rejected"
  },
  "_meta": {
    "not_found": false
  }
}
```

```text expect=disgenet-rejected-key contains
"message":
configured DISGENET_API_KEY credential was rejected or does not have access
```

```text expect=disgenet-rejected-key not-contains
fixture-disgenet-rejected-key-not-a-secret
```

```bash run id=disgenet-rejected-key-stderr exit=1 stream=stderr
biomcp --json get gene BRAF disgenet
```

```text expect=disgenet-rejected-key-stderr not-contains
fixture-disgenet-rejected-key-not-a-secret
```
