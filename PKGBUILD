# Maintainer: Your Name <your@email.tld>
pkgname=cookie-clicker-cli
pkgver=0.1.0
pkgrel=1
pkgdesc="A Cookie Clicker-style CLI game with gambling minigames – written in Rust"
arch=('x86_64' 'aarch64')
url="https://github.com/m4rcel-lol/cli-clicker-gambling"
license=('MIT')
depends=()
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/heads/main.tar.gz")
sha256sums=('SKIP')

prepare() {
    cd "$srcdir/cli-clicker-gambling-main"
    # Ensure Cargo.lock is present for --locked build
    cargo fetch --locked
}

build() {
    cd "$srcdir/cli-clicker-gambling-main"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR="$srcdir/target"
    cargo build --release --locked
}

check() {
    cd "$srcdir/cli-clicker-gambling-main"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR="$srcdir/target"
    cargo test --release --locked
}

package() {
    cd "$srcdir/cli-clicker-gambling-main"
    install -Dm755 "$srcdir/target/release/cookie_clicker" \
        "$pkgdir/usr/bin/cookie-clicker"
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
