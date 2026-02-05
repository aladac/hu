List pods in the EKS cluster.

## Usage

```bash
hu eks list                          # Default namespace
hu eks list -n production            # Specific namespace
hu eks list -A                       # All namespaces
hu eks list -c my-cluster            # Specific context
hu eks list --json                   # JSON output
```

## Options

| Flag | Description |
|------|-------------|
| `-n, --namespace` | Namespace to list pods from |
| `-A, --all-namespaces` | List pods from all namespaces |
| `-c, --context` | Kubeconfig context to use |
| `--json` | Output as JSON |

## Related Commands

| Command | Purpose |
|---------|---------|
| `hu eks exec <POD>` | Shell into a pod |
| `hu eks logs <POD>` | Tail pod logs |
