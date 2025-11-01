#!/usr/bin/env python3
"""
GitHub Release Creation Script for Reratui
This script helps create a GitHub release
"""

import argparse
import re
import subprocess
import sys
import webbrowser
from pathlib import Path

# Configuration
REPO = "sabry-awad97/reratui"


def get_current_version(root_dir: Path) -> str:
    """Get current version from workspace Cargo.toml"""
    cargo_toml = root_dir / "Cargo.toml"
    content = cargo_toml.read_text(encoding="utf-8")
    
    match = re.search(r'version\s*=\s*"(\d+\.\d+\.\d+)"', content)
    if not match:
        raise ValueError("Could not find version in Cargo.toml")
    
    return match.group(1)


def print_colored(text, color="white"):
    """Print colored text to console"""
    colors = {
        "green": "\033[92m",
        "yellow": "\033[93m",
        "cyan": "\033[96m",
        "red": "\033[91m",
        "reset": "\033[0m",
    }
    color_code = colors.get(color, colors["reset"])
    print(f"{color_code}{text}{colors['reset']}")


def check_gh_cli():
    """Check if GitHub CLI is installed"""
    try:
        subprocess.run(
            ["gh", "--version"],
            capture_output=True,
            check=True,
        )
        return True
    except (subprocess.CalledProcessError, FileNotFoundError):
        return False


def create_release_with_gh(version: str, title: str, notes_file: str):
    """Create release using GitHub CLI"""
    print_colored("GitHub CLI found. Creating release...", "cyan")
    
    try:
        subprocess.run(
            [
                "gh",
                "release",
                "create",
                f"v{version}",
                "--title",
                title,
                "--notes-file",
                notes_file,
            ],
            check=True,
        )
        print()
        print_colored("✅ Release created successfully!", "green")
        print_colored(
            f"View at: https://github.com/{REPO}/releases/tag/v{version}", "cyan"
        )
        return True
    except subprocess.CalledProcessError:
        print()
        print_colored("❌ Failed to create release", "red")
        return False


def manual_release_instructions(version: str, title: str, notes_file: str):
    """Show manual release instructions"""
    print_colored("GitHub CLI (gh) not found.", "yellow")
    print()
    print_colored("Option 1: Install GitHub CLI", "cyan")
    print("  pip install gh")
    print("  Or download from: winget install --id GitHub.cli")
    print("  Then run this script again")
    print()
    print_colored("Option 2: Create release manually", "cyan")
    print(f"  1. Go to: https://github.com/{REPO}/releases/new?tag=v{version}")
    print(f"  2. Set title: {title}")
    print(f"  3. Copy content from: {notes_file}")
    print("  4. Click 'Publish release'")
    print()
    
    # Ask if user wants to open browser
    try:
        response = (
            input("Open GitHub releases page in browser? (y/n): ").strip().lower()
        )
        if response in ("y", "yes"):
            url = f"https://github.com/{REPO}/releases/new?tag=v{version}"
            webbrowser.open(url)
            print_colored("Opening browser...", "green")
    except KeyboardInterrupt:
        print()
        print("Cancelled.")


def main():
    """Main function"""
    parser = argparse.ArgumentParser(
        description="Create GitHub release for Reratui",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python create_release.py                           # Use current version
  python create_release.py --version 0.3.0           # Specific version
  python create_release.py --notes custom_notes.md   # Custom notes file
        """
    )
    
    parser.add_argument(
        "--version",
        help="Version to release (default: auto-detect from Cargo.toml)"
    )
    parser.add_argument(
        "--title",
        help="Release title (default: auto-generated)"
    )
    parser.add_argument(
        "--notes",
        help="Release notes file (default: RELEASE_NOTES_vX.Y.Z.md)"
    )
    
    args = parser.parse_args()
    
    # Get root directory
    root_dir = Path(__file__).parent.parent
    
    # Determine version
    if args.version:
        version = args.version
    else:
        version = get_current_version(root_dir)
    
    # Determine title
    if args.title:
        title = args.title
    else:
        title = f"v{version}"
    
    # Determine notes file
    if args.notes:
        notes_file = args.notes
    else:
        notes_file = f"RELEASE_NOTES_v{version}.md"
    
    print_colored(f"Creating GitHub Release for v{version}", "green")
    print()
    
    # Check if notes file exists
    notes_path = root_dir / notes_file
    if not notes_path.exists():
        print_colored(f"❌ Release notes file not found: {notes_file}", "red")
        print_colored("   Create it first or specify --notes", "yellow")
        sys.exit(1)
    
    # Try to create release with GitHub CLI
    if check_gh_cli():
        create_release_with_gh(version, title, notes_file)
    else:
        manual_release_instructions(version, title, notes_file)
    
    print()
    print_colored(f"Release notes are in: {notes_file}", "cyan")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print()
        print_colored("Cancelled by user.", "yellow")
        sys.exit(0)
