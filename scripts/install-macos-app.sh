#!/usr/bin/env bash
#
# Create a Spotlight/Launchpad-discoverable "BreakTime.app" in ~/Applications on
# macOS.
#
# WHY THIS IS NEEDED:
#
# When built through this repo's Nix setup, break-time is a native macOS
# (Cocoa) application, and `nix profile add` installs a proper app bundle at
# ~/.nix-profile/Applications/BreakTime.app.  However, macOS Spotlight and
# Launchpad will NOT find that bundle: they don't index apps that live in the
# Nix store, and they refuse to index symlinked .app bundles (so simply
# symlinking it into ~/Applications does not work either).  See
# https://github.com/NixOS/nix/issues/7055 for more info.
#
# This script works around that by first assuming the user installed break-time
# with `nix profile add`, and then by dropping a tiny "trampoline" app into
# ~/Applications/BreakTime.app.  It is a real (non-symlink) .app directory --
# which Spotlight does index -- containing:
#
#   * the installed bundle's Info.plist and icon (so the name and menu-bar
#     behaviour match), and
#   * an executable that is just a symlink to the real binary at
#     ~/.nix-profile/bin/break-time.
#
# Only the .app DIRECTORY itself must be real for Spotlight to index it; a
# symlinked executable inside it is fine (verified empirically).  The
# executable must NOT be a shell script that exec's the real binary: launching
# through LaunchServices with that extra sh->exec hop breaks NSStatusItem
# visibility on current macOS -- the app runs, but its menu-bar item never
# shows up.  A symlink avoids the hop entirely.
#
# Because the symlink points at the stable ~/.nix-profile path (rather than a
# specific /nix/store/... path), this only has to be run ONCE: the trampoline
# keeps working across `nix profile upgrade`.  Re-run it only if you delete
# ~/Applications/BreakTime.app or install break-time into a different Nix
# profile.
#
# PREREQUISITE: install break-time into your Nix profile first, from a checkout
# of this repo:
#
#   $ nix profile add --file ./nixpkgs.nix break-time
#
# USAGE:
#
#   $ ./scripts/install-macos-app.sh
#
# To undo, just remove the trampoline:
#
#   $ rm -rf ~/Applications/BreakTime.app
#
# STARTING AUTOMATICALLY AT LOGIN: after running this script, add the
# trampoline as a login item yourself:
#
#   System Settings -> General -> Login Items & Extensions ->
#     "Open at Login" -> "+" -> select ~/Applications/BreakTime.app
#
# NOTE: If you manage your Mac declaratively with nix-darwin or home-manager,
# prefer the `mac-app-util` module (https://github.com/hraban/mac-app-util),
# which does the same trampolining automatically on every rebuild.  This script
# is the low-tech, no-extra-dependencies alternative for a single app.

set -e

# Stable, upgrade-proof locations of the Nix-profile-installed break-time.
profile_app="$HOME/.nix-profile/Applications/BreakTime.app"
profile_bin="$HOME/.nix-profile/bin/break-time"

# Where the trampoline app is created.
dest="$HOME/Applications/BreakTime.app"

if [ ! -e "$profile_bin" ]; then
  echo "error: $profile_bin not found." >&2
  echo "Install break-time into your Nix profile first: nix profile add --file ./nixpkgs.nix break-time" >&2
  exit 1
fi

mkdir -p "$HOME/Applications"
rm -rf "$dest"
mkdir -p "$dest/Contents/MacOS" "$dest/Contents/Resources"

# Reuse the installed bundle's metadata and icon so the name/icon stay in sync
# with whatever version is currently installed.  (Files copied out of the Nix
# store are read-only, so make them writable.)
cp "$profile_app/Contents/Info.plist" "$dest/Contents/Info.plist"
cp "$profile_app/Contents/Resources/break-time.icns" "$dest/Contents/Resources/break-time.icns"
chmod u+w "$dest/Contents/Info.plist" "$dest/Contents/Resources/break-time.icns"

# The executable is a symlink to the stable profile binary — NOT a wrapper
# script; see the header comment for why (sh->exec breaks the menu-bar item).
ln -s "$profile_bin" "$dest/Contents/MacOS/BreakTime"

# Register with LaunchServices right away so it shows up in Spotlight without
# having to wait for the next indexing pass.
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$dest" || true

echo "Created $dest -> $profile_bin"
