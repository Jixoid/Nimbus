#!/usr/bin/env bash
set -euo pipefail

CRATE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_DIR="$CRATE_DIR/target/android-apk"
JNI_DIR="$OUT_DIR/jniLibs"
ABI="${1:-arm64-v8a}"
API_LEVEL=29

export ANDROID_HOME="${ANDROID_HOME:-/home/alforce/Android/Sdk}"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/28.2.13676358}"

BUILD_TOOLS="$ANDROID_HOME/build-tools/36.0.0"
PLATFORM_JAR="$ANDROID_HOME/platforms/android-36/android.jar"
export ANDROID_JAR="$PLATFORM_JAR"

AAPT2="$BUILD_TOOLS/aapt2"
ZIPALIGN="$BUILD_TOOLS/zipalign"
APKSIGNER="$BUILD_TOOLS/apksigner"

mkdir -p "$OUT_DIR" "$JNI_DIR"

echo "==> [1/5] Building libnimbus_demo_android.so via cargo-ndk..."
cd "$CRATE_DIR"
cargo ndk -t "$ABI" -P "$API_LEVEL" -o "$JNI_DIR" build --release

SO_PATH="$JNI_DIR/$ABI/libnimbus_demo_android.so"
if [[ ! -f "$SO_PATH" ]]; then
    echo "ERROR: Expected $SO_PATH not found" >&2
    exit 1
fi

READELF="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-readelf"
if "$READELF" -d "$SO_PATH" | grep -q "libc++_shared.so"; then
    case "$ABI" in
        arm64-v8a) NDK_TRIPLE="aarch64-linux-android" ;;
        armeabi-v7a) NDK_TRIPLE="arm-linux-androideabi" ;;
        *) NDK_TRIPLE="" ;;
    esac
    LIBCXX="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/$NDK_TRIPLE/libc++_shared.so"
    if [[ -n "$NDK_TRIPLE" && -f "$LIBCXX" ]]; then
        echo "==> Copying libc++_shared.so into $JNI_DIR/$ABI/"
        cp -f "$LIBCXX" "$JNI_DIR/$ABI/libc++_shared.so"
    fi
fi

echo "==> [2/5] Linking AndroidManifest.xml with aapt2..."
UNALIGNED_APK="$OUT_DIR/nimbus_demo_android.unaligned.apk"
ALIGNED_APK="$OUT_DIR/nimbus_demo_android.aligned.apk"
FINAL_APK="$OUT_DIR/nimbus_demo_android.apk"
rm -f "$UNALIGNED_APK" "$ALIGNED_APK" "$FINAL_APK"

"$AAPT2" link \
    -I "$PLATFORM_JAR" \
    --manifest "$CRATE_DIR/AndroidManifest.xml" \
    -o "$UNALIGNED_APK"

echo "==> [3/5] Packaging uncompressed lib/$ABI/*.so into APK..."
python3 - "$UNALIGNED_APK" "$JNI_DIR/$ABI" "$ABI" <<'PY'
import os
import sys
import zipfile

apk_path, so_dir, abi = sys.argv[1], sys.argv[2], sys.argv[3]
with zipfile.ZipFile(apk_path, "a", compression=zipfile.ZIP_STORED) as zf:
    for fname in sorted(os.listdir(so_dir)):
        if fname.endswith(".so"):
            src = os.path.join(so_dir, fname)
            arc = f"lib/{abi}/{fname}"
            zf.write(src, arc, compress_type=zipfile.ZIP_STORED)
            print(f"    added {arc} (stored)")
PY

echo "==> [4/5] Aligning APK..."
"$ZIPALIGN" -f -p 4 "$UNALIGNED_APK" "$ALIGNED_APK"

KEYSTORE="$OUT_DIR/debug.keystore"
if [[ ! -f "$KEYSTORE" ]]; then
    echo "==> Generating debug keystore..."
    keytool -genkeypair -v \
        -keystore "$KEYSTORE" \
        -storepass android \
        -alias androiddebugkey \
        -keypass android \
        -keyalg RSA \
        -keysize 2048 \
        -validity 10000 \
        -dname "CN=Android Debug,O=Android,C=US"
fi

echo "==> [5/5] Signing APK..."
"$APKSIGNER" sign \
    --ks "$KEYSTORE" \
    --ks-pass pass:android \
    --ks-key-alias androiddebugkey \
    --key-pass pass:android \
    --out "$FINAL_APK" \
    "$ALIGNED_APK"

rm -f "$UNALIGNED_APK" "$ALIGNED_APK"
echo "==> Done: $FINAL_APK"
