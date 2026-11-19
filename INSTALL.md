# Install Habits

Install Habits from PrimitiveOS Store on a Kindle with Primitive Launcher. The Store listing has the
package requirements and SHA-256. A local build uses the native ARMHF target.

```sh
cargo zigbuild --locked --release -p habits-kindle --target armv7-unknown-linux-musleabihf
sh scripts/stage.sh armhf
python3 ../doannt-kindle-build/scripts/package.py \
  --id primitive-habits --dir habits --version 0.1.0 --arch armhf \
  --stage build/stage --license LICENSE --docs-license LICENSE-DOCS --install INSTALL.md --out dist
```
