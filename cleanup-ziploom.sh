#!/bin/bash
# cleanup-ziploom.sh — hapus semua ZipLoom dari MacBook
echo "🧹 Membersihkan semua ZipLoom..."

rm -f ~/Downloads/ZipLoom_V1_Final.zip 2>/dev/null
rm -rf ~/Documents/ziploom-tauri 2>/dev/null
rm -rf ~/Desktop/ZipLoom* 2>/dev/null
rm -rf ~/Downloads/ZipLoom* ~/Downloads/ziploom* 2>/dev/null

echo "✅ Selesai! Semua ZipLoom udah dibersihin."
