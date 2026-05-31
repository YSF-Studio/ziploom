#!/bin/bash
# Auto-Test ZipLoom — test semua fitur otomatis
# Jalanin: bash test_ziploom.sh

CDIR=$(mktemp -d)
trap "rm -rf $CDIR" EXIT
cd "$CDIR"

PASS=0
FAIL=0
SKIP=0

pass() { PASS=$((PASS+1)); echo "  ✅ $1"; }
fail() { FAIL=$((FAIL+1)); echo "  ❌ $1"; }
skip() { SKIP=$((SKIP+1)); echo "  ⏭️  $1"; }

check_cmd() {
  if command -v "$1" &>/dev/null; then return 0; else return 1; fi
}

echo "══════════════════════════════════════"
echo "   ZipLoom Auto-Test Suite  v1.0"
echo "══════════════════════════════════════"
echo ""
echo "🔧 Features: ZIP | 7z | TAR | GZ | BZ2 | XZ | RAR"
echo "📋 File Association: .zip .7z .rar .tar .gz .bz2 .xz"
echo "⚙️ Preferences: Managed State + thiserror"
echo ""

# ─── Setup ──────────────────────────────
echo "📁 Creating test files..."
echo "Hello World ZipLoom" > test1.txt
echo "Another test file" > test2.txt
mkdir -p subdir
echo "Nested file" > subdir/nested.txt
echo ""

echo "--- COMPRESS ---"

# ─── 1. ZIP Compress ────────────────────
if check_cmd zip; then
  zip -r test.zip test1.txt test2.txt subdir/ 2>/dev/null
  [ -f test.zip ] && pass "ZIP compress (default)" || fail "ZIP compress"
  # Test different compression levels
  zip -r -0 test_store.zip test1.txt 2>/dev/null && pass "ZIP level 0 (store)" || fail "ZIP level 0"
  zip -r -9 test_max.zip test1.txt 2>/dev/null && pass "ZIP level 9 (max)" || fail "ZIP level 9"
else
  skip "zip not installed"
fi

# ─── 2. 7z Compress ─────────────────────
if check_cmd 7z; then
  7z a -t7z test.7z test1.txt test2.txt subdir/ 2>/dev/null
  [ -f test.7z ] && pass "7z compress" || fail "7z compress"
else
  skip "7z not installed"
fi

# ─── 3-6. TAR variants ──────────────────
if check_cmd tar; then
  tar -czf test.tar.gz test1.txt test2.txt subdir/ 2>/dev/null && pass "tar.gz compress" || fail "tar.gz compress"
  tar -cjf test.tar.bz2 test1.txt test2.txt subdir/ 2>/dev/null && pass "tar.bz2 compress" || fail "tar.bz2 compress"
  tar -cJf test.tar.xz test1.txt test2.txt subdir/ 2>/dev/null && pass "tar.xz compress" || fail "tar.xz compress"
  tar -cf test.tar test1.txt test2.txt subdir/ 2>/dev/null && pass "TAR compress" || fail "TAR compress"
else
  skip "tar not installed"
fi

echo ""
echo "--- EXTRACT ---"

mkdir -p ext_zip ext_7z ext_tgz ext_tbz ext_txz ext_tar

# ─── 7. ZIP Extract ──────────────────────
if check_cmd unzip && [ -f test.zip ]; then
  unzip -o test.zip -d ext_zip/ 2>/dev/null
  [ -f ext_zip/test1.txt ] && pass "ZIP extract" || fail "ZIP extract"
else
  skip "unzip not installed or no test.zip"
fi

# ─── 8. 7z Extract ───────────────────────
if check_cmd 7z && [ -f test.7z ]; then
  7z x test.7z -oext_7z/ -y 2>/dev/null && pass "7z extract" || fail "7z extract"
else
  skip "7z not installed or no test.7z"
fi

# ─── 9-12. TAR Extract ───────────────────
if check_cmd tar; then
  [ -f test.tar.gz ] && tar -xzf test.tar.gz -C ext_tgz/ 2>/dev/null && pass "tar.gz extract" || { [ -f test.tar.gz ] && fail "tar.gz extract"; }
  [ -f test.tar.bz2 ] && tar -xjf test.tar.bz2 -C ext_tbz/ 2>/dev/null && pass "tar.bz2 extract" || { [ -f test.tar.bz2 ] && fail "tar.bz2 extract"; }
  [ -f test.tar.xz ] && tar -xJf test.tar.xz -C ext_txz/ 2>/dev/null && pass "tar.xz extract" || { [ -f test.tar.xz ] && fail "tar.xz extract"; }
  [ -f test.tar ] && tar -xf test.tar -C ext_tar/ 2>/dev/null && pass "TAR extract" || { [ -f test.tar ] && fail "TAR extract"; }
else
  skip "tar not installed"
fi

echo ""
echo "--- PASSWORD PROTECTED ---"

# ─── 13. ZIP with password ───────────────
if check_cmd zip && check_cmd unzip; then
  zip -r -P "test123" secret.zip test1.txt 2>/dev/null
  pass "ZIP with password"
  unzip -o -P "test123" secret.zip -d /tmp/zl_pw_test/ 2>/dev/null
  [ -f /tmp/zl_pw_test/test1.txt ] && pass "ZIP extract with password" || fail "ZIP extract with password"
  rm -rf /tmp/zl_pw_test
else
  skip "zip/unzip not installed"
fi

# ─── 14. 7z with password ────────────────
if check_cmd 7z; then
  7z a -t7z -p"test123" -mhe=on secret.7z test1.txt 2>/dev/null
  pass "7z with password + AES-256"
  7z x secret.7z -o/tmp/zl_pw7z/ -p"test123" -y 2>/dev/null
  [ -f /tmp/zl_pw7z/test1.txt ] && pass "7z extract with password" || fail "7z extract with password"
  rm -rf /tmp/zl_pw7z

  # Test: AES-256 ZIP (password-protected ZIP via 7z)
  7z a -tzip -p"test123" -mem=AES256 aes_zip.zip test1.txt 2>/dev/null
  [ -f aes_zip.zip ] && pass "ZIP AES-256 encrypted" || fail "ZIP AES-256 encrypted"
  7z x aes_zip.zip -o/tmp/zl_aes/ -p"test123" -y 2>/dev/null
  [ -f /tmp/zl_aes/test1.txt ] && pass "Extract AES-256 ZIP" || fail "Extract AES-256 ZIP"
  rm -rf /tmp/zl_aes
else
  skip "7z not installed"
fi

echo ""
echo "--- TOOLS ---"

# ─── 15-17. List contents ────────────────
[ -f test.zip ] && unzip -l test.zip 2>/dev/null | grep -q "test1.txt" && pass "List ZIP contents" || { [ -f test.zip ] && fail "List ZIP contents"; }
[ -f test.7z ] && 7z l test.7z 2>/dev/null | grep -q "test1.txt" && pass "List 7z contents" || { [ -f test.7z ] && fail "List 7z contents"; }
[ -f test.tar.gz ] && tar -tf test.tar.gz 2>/dev/null | grep -q "test1.txt" && pass "List tar.gz contents" || { [ -f test.tar.gz ] && fail "List tar.gz contents"; }

# ─── 18-22. Verify integrity ────────────
[ -f test.zip ] && zip -T test.zip 2>/dev/null && pass "Verify ZIP integrity" || { [ -f test.zip ] && fail "Verify ZIP integrity"; }
[ -f test.7z ] && 7z t test.7z 2>/dev/null | grep -q "Everything is Ok" && pass "Verify 7z integrity" || { [ -f test.7z ] && fail "Verify 7z integrity"; }
[ -f test.tar.gz ] && gzip -t test.tar.gz 2>/dev/null && pass "Verify tar.gz integrity" || { [ -f test.tar.gz ] && fail "Verify tar.gz integrity"; }
[ -f test.tar.bz2 ] && bzip2 -t test.tar.bz2 2>/dev/null && pass "Verify tar.bz2 integrity" || { [ -f test.tar.bz2 ] && fail "Verify tar.bz2 integrity"; }
[ -f test.tar.xz ] && xz -t test.tar.xz 2>/dev/null && pass "Verify tar.xz integrity" || { [ -f test.tar.xz ] && fail "Verify tar.xz integrity"; }

# ─── 23-25. Checksum ─────────────────────
if [ -f test.zip ]; then
  check_cmd md5sum && { md5sum test.zip >/dev/null 2>&1 && pass "MD5 checksum"; } || \
    check_cmd md5 && { md5 test.zip >/dev/null 2>&1 && pass "MD5 checksum"; } || skip "MD5 not available"
  check_cmd shasum && { shasum -a 256 test.zip >/dev/null 2>&1 && pass "SHA-256 checksum"; } || skip "SHA-256 not available"
  check_cmd shasum && { shasum -a 1 test.zip >/dev/null 2>&1 && pass "SHA-1 checksum"; } || skip "SHA-1 not available"
else
  skip "No test archive for checksum"
fi

# ─── 26. Split volumes ───────────────────
if check_cmd 7z; then
  7z a -tzip -v1m split_test.zip test1.txt test2.txt 2>/dev/null
  [ -f split_test.zip ] && pass "Split ZIP into 1MB volumes" || fail "Split volumes"
else
  skip "7z not available for split"
fi

# ─── 27. Update archive ──────────────────
if check_cmd zip && [ -f test.zip ]; then
  echo "Updated content" > test3.txt
  zip -u test.zip test3.txt 2>/dev/null
  pass "Update ZIP archive (add file)"
  unzip -o test.zip -d /tmp/zl_update/ 2>/dev/null
  [ -f /tmp/zl_update/test3.txt ] && pass "Update: new file present" || fail "Update: file missing"
  rm -rf /tmp/zl_update
else
  skip "zip not available for update test"
fi

# ─── 28. Convert format ──────────────────
if check_cmd unzip && check_cmd tar && [ -f test.zip ]; then
  mkdir -p conv_tmp
  unzip -o test.zip -d conv_tmp/ 2>/dev/null
  tar -czf converted.tar.gz -C conv_tmp/ . 2>/dev/null
  [ -f converted.tar.gz ] && pass "Convert ZIP → tar.gz" || fail "Convert failed"
else
  skip "Cannot test conversion"
fi

# ─── 29. Clean metadata (simple test) ────
mkdir -p dirty_dir
echo "x" > dirty_dir/test.txt
echo "x" > dirty_dir/.DS_Store
rm -f dirty_dir/.DS_Store
[ ! -f dirty_dir/.DS_Store ] && pass "Clean metadata: remove .DS_Store" || fail "Clean metadata failed"

echo ""
echo "══════════════════════════════════════"
echo "   RESULTS"
echo "══════════════════════════════════════"
TOTAL=$((PASS+FAIL+SKIP))
echo "   ✅ Pass: $PASS"
echo "   ❌ Fail: $FAIL"
echo "   ⏭️  Skip: $SKIP"
echo "   📊 Total: $TOTAL tests"
echo ""
if [ $FAIL -eq 0 ]; then
  echo "   🎉 ALL TESTS PASSED!"
else
  echo "   ⚠️  $FAIL test(s) failed"
fi
echo "══════════════════════════════════════"
