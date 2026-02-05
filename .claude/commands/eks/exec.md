Execute a command in an EKS pod (interactive shell by default).

## Usage

```bash
hu eks exec my-pod                         # Interactive shell (/bin/sh)
hu eks exec my-pod -n production           # Specific namespace
hu eks exec my-pod -c my-container         # Specific container
hu eks exec my-pod -- ls -la               # Run specific command
hu eks exec my-pod -- rails console        # Rails console
```

## Arguments

| Arg | Description |
|-----|-------------|
| `POD` | Pod name (required) |
| `COMMAND...` | Command to run after `--` (default: /bin/sh) |

## Options

| Flag | Description |
|------|-------------|
| `-n, --namespace` | Namespace |
| `-c, --container` | Container name (if pod has multiple) |
| `--context` | Kubeconfig context to use |

## Safety

READ-ONLY operations recommended. Use `-e dev` only for EKS testing.
