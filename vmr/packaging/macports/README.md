# vmr MacPorts port source

This repository is a [MacPorts](https://www.macports.org/) port source for
[`vmr`](https://github.com/gvcgo/version-manager). It is rendered and pushed by the
vmr release workflow; do not edit `devel/vmr/Portfile` by hand.

MacPorts has no Homebrew-style tap mechanism, so the source is added once as a
local `file://` tree:

```bash
sudo mkdir -p /opt/local/var/macports/sources/github.com/gvcgo
sudo git clone https://github.com/gvcgo/vmr-macports \
  /opt/local/var/macports/sources/github.com/gvcgo/vmr-macports
echo "file:///opt/local/var/macports/sources/github.com/gvcgo/vmr-macports [nosync]" \
  | sudo tee -a /opt/local/etc/macports/sources.conf
sudo portindex /opt/local/var/macports/sources/github.com/gvcgo/vmr-macports
sudo port install vmr
```

Use your own MacPorts prefix instead of `/opt/local` if it differs (`port -v`
prints the prefix). The `[nosync]` flag is required because the checkout lives
outside MacPorts' own source cache; run `portindex` manually after pulling.

To update later:

```bash
sudo git -C /opt/local/var/macports/sources/github.com/gvcgo/vmr-macports pull
sudo portindex /opt/local/var/macports/sources/github.com/gvcgo/vmr-macports
sudo port upgrade vmr
```

The port installs the same prebuilt macOS binary used by the Homebrew formula and
the release archives. It does not compile anything locally. SDK versions are
managed by vmr itself under `~/.vmr`; see the
[main README](https://github.com/gvcgo/version-manager#readme).

This source is not part of the official `macports-ports` tree. If a conflicting
`vmr` port is added upstream later, remove this source from
`/opt/local/etc/macports/sources.conf`.
