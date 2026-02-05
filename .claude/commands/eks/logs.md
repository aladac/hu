Tail logs from an EKS pod.

## Usage

```bash
hu eks logs my-pod                         # Recent logs
hu eks logs my-pod -f                      # Follow (stream)
hu eks logs my-pod --tail 100              # Last 100 lines
hu eks logs my-pod --previous              # Previous container instance
hu eks logs my-pod -n production           # Specific namespace
hu eks logs my-pod -c my-container         # Specific container
```

## Arguments

| Arg | Description |
|-----|-------------|
| `POD` | Pod name (required) |

## Options

| Flag | Description |
|------|-------------|
| `-n, --namespace` | Namespace |
| `-c, --container` | Container name (if pod has multiple) |
| `-f, --follow` | Follow log output |
| `--previous` | Logs from previous container instance |
| `--tail` | Number of lines from the end |
| `--context` | Kubeconfig context to use |
