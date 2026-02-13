# Ideas for hu CLI

## Network Switching (`hu net`)

macOS network management commands for switching between Wi-Fi and iPhone USB tethering.

### Use Cases

1. **Wi-Fi only** - Normal home/office use
2. **Split mode** - iPhone USB for internet (bypass ISP), Wi-Fi for LAN (local devices, NAS, printers)
3. **iPhone only** - Mobile-only, Wi-Fi disabled (travel, security)

### Commands

```bash
hu net config    # Show network status
hu net switch    # Cycle through modes: wifi → split → iphone → wifi
hu net wifi      # Switch to Wi-Fi for all traffic
hu net split     # iPhone for internet, Wi-Fi for LAN
hu net iphone    # iPhone for all traffic (disables Wi-Fi)
```

### Implementation Details

#### macOS Network Interfaces

| Service | Interface | Typical IP |
|---------|-----------|------------|
| Wi-Fi | en0 | 192.168.x.x |
| iPhone USB | en7 | 172.20.10.x |
| Thunderbolt Bridge | bridge0 | - |

#### Key Commands

```bash
# List services and order
networksetup -listnetworkserviceorder

# Change service priority (first = default route)
sudo networksetup -ordernetworkservices "iPhone USB" "Wi-Fi" "Thunderbolt Bridge"

# Toggle Wi-Fi power
sudo networksetup -setairportpower en0 on
sudo networksetup -setairportpower en0 off

# Check Wi-Fi power state
networksetup -getairportpower en0

# Get interface IP
ifconfig en0 | awk '/inet / {print $2}'

# Get default route
route -n get default | grep -E 'gateway|interface'

# Check interface status
ifconfig en7 | grep status
```

#### Mode Detection Logic

```
if default_interface == "en7" (iPhone):
    if wifi_power == "Off":
        mode = "iphone"
    else:
        mode = "split"
else:
    mode = "wifi"
```

#### Mode Switching Logic

| Mode | Service Order | Wi-Fi Power |
|------|---------------|-------------|
| wifi | Wi-Fi, iPhone USB, TB | On |
| split | iPhone USB, Wi-Fi, TB | On |
| iphone | iPhone USB, Wi-Fi, TB | Off |

### Output Format

#### `hu net config`

```
MODE|wifi
DEFAULT_GW|192.168.0.1
DEFAULT_IFACE|en0
WIFI_POWER|On

TABLE_START
Service|Interface|IP|Status|Priority
Wi-Fi|en0|192.168.0.17|active *|1
iPhone USB|en7|172.20.10.3|active|2
Thunderbolt Bridge|bridge0|-|down|3
TABLE_END
```

#### `hu net switch`

```
MODE|split
DESC|iPhone USB for internet, Wi-Fi for LAN
DEFAULT_IFACE|en7
DEFAULT_GW|172.20.10.1

TEST_START
ping|8.8.8.8|ok
public_ip|188.146.164.44|ok
TEST_END
```

### Connectivity Tests

After switching, verify:

1. **Ping** - `ping -c 1 -t 2 8.8.8.8`
2. **Public IP** - `curl -s https://api.ipify.org`
3. **DNS** - `nslookup google.com`
4. **LAN** (split mode) - `ping -c 1 192.168.0.1`

### Notes

- Requires `sudo` for `networksetup` commands
- iPhone must have Personal Hotspot enabled and be connected via USB
- macOS automatically routes LAN traffic via Wi-Fi even when iPhone is default (no static routes needed)
- Service order persists across reboots

### Reference Implementation

See bash scripts in `~/.claude/commands/net/`:
- `config.sh` - Network status display
- `switch.sh` - Mode switching with validation
