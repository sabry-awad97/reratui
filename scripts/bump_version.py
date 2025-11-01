#!/usr/bin/env python3
"""
Version Bump Script for Reratui
Automatically bumps version across all crates and creates git tag
"""

import argparse
import re
import subprocess
import sys
from pathlib import Path
from typing import List, Tuple


# Color codes for terminal output
class Colors:
    GREEN = "\033[92m"
    YELLOW = "\033[93m"
    CYAN = "\033[96m"
    RED = "\033[91m"
    BOLD = "\033[1m"
    RESET = "\033[0m"


def print_colored(text: str, color: str = ""):
    """Print colored text to console"""
    print(f"{color}{text}{Colors.RESET}")


def print_step(step: str):
    """Print a step header"""
    print_colored(f"\n{'='*60}", Colors.CYAN)
    print_colored(f"  {step}", Colors.BOLD + Colors.CYAN)
    print_colored(f"{'='*60}", Colors.CYAN)


def parse_version(version: str) -> Tuple[int, int, int]:
    """Parse version string into (major, minor, patch)"""
    match = re.match(r"(\d+)\.(\d+)\.(\d+)", version)
    if not match:
        raise ValueError(f"Invalid version format: {version}")
    return tuple(map(int, match.groups()))


def format_version(major: int, minor: int, patch: int) -> str:
    """Format version tuple as string"""
    return f"{major}.{minor}.{patch}"


def bump_version(current: str, bump_type: str) -> str:
    """Bump version based on type (major, minor, patch)"""
    major, minor, patch = parse_version(current)
    
    if bump_type == "major":
        return format_version(major + 1, 0, 0)
    elif bump_type == "minor":
        return format_version(major, minor + 1, 0)
    elif bump_type == "patch":
        return format_version(major, minor, patch + 1)
    else:
        raise ValueError(f"Invalid bump type: {bump_type}")


def get_current_version(root_dir: Path) -> str:
    """Get current version from workspace Cargo.toml"""
    cargo_toml = root_dir / "Cargo.toml"
    content = cargo_toml.read_text(encoding="utf-8")
    
    match = re.search(r'version\s*=\s*"(\d+\.\d+\.\d+)"', content)
    if not match:
        raise ValueError("Could not find version in Cargo.toml")
    
    return match.group(1)


def update_file_version(file_path: Path, old_version: str, new_version: str) -> bool:
    """Update version in a single file"""
    try:
        content = file_path.read_text(encoding="utf-8")
        
        # Replace version = "x.y.z"
        new_content = re.sub(
            rf'version\s*=\s*"{re.escape(old_version)}"',
            f'version = "{new_version}"',
            content
        )
        
        if new_content != content:
            file_path.write_text(new_content, encoding="utf-8")
            return True
        return False
    except Exception as e:
        print_colored(f"  ❌ Error updating {file_path}: {e}", Colors.RED)
        return False


def update_crate_versions(root_dir: Path, old_version: str, new_version: str):
    """Update version in all crate Cargo.toml files"""
    print_step("Updating Crate Versions")
    
    crates = [
        "reratui-panic",
        "reratui-core",
        "reratui-macro",
        "reratui-hooks",
        "reratui-ratatui",
        "reratui-runtime",
        "reratui",
        "reratui-charts",
        "reratui-forms",
        "reratui-icons",
        "reratui-router",
    ]
    
    updated = []
    for crate in crates:
        cargo_toml = root_dir / "crates" / crate / "Cargo.toml"
        if cargo_toml.exists():
            if update_file_version(cargo_toml, old_version, new_version):
                print_colored(f"  ✅ {crate}", Colors.GREEN)
                updated.append(crate)
            else:
                print_colored(f"  ⚠️  {crate} (no changes)", Colors.YELLOW)
    
    return updated


def update_workspace_version(root_dir: Path, old_version: str, new_version: str):
    """Update workspace version in root Cargo.toml"""
    print_step("Updating Workspace Version")
    
    cargo_toml = root_dir / "Cargo.toml"
    if update_file_version(cargo_toml, old_version, new_version):
        print_colored("  ✅ Workspace Cargo.toml", Colors.GREEN)
    else:
        print_colored("  ⚠️  No changes needed", Colors.YELLOW)


def run_command(cmd: List[str], cwd: Path, check: bool = True) -> subprocess.CompletedProcess:
    """Run a shell command"""
    try:
        result = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=True,
            text=True,
            check=check,
        )
        return result
    except subprocess.CalledProcessError as e:
        print_colored(f"  ❌ Command failed: {' '.join(cmd)}", Colors.RED)
        print_colored(f"  Error: {e.stderr}", Colors.RED)
        raise


def verify_build(root_dir: Path):
    """Verify that the project builds"""
    print_step("Verifying Build")
    
    print_colored("  Running: cargo build --workspace --lib", Colors.CYAN)
    result = run_command(["cargo", "build", "--workspace", "--lib"], root_dir)
    
    if result.returncode == 0:
        print_colored("  ✅ Build successful", Colors.GREEN)
    else:
        print_colored("  ❌ Build failed", Colors.RED)
        sys.exit(1)


def run_tests(root_dir: Path):
    """Run tests"""
    print_step("Running Tests")
    
    print_colored("  Running: cargo test --workspace --lib", Colors.CYAN)
    result = run_command(["cargo", "test", "--workspace", "--lib"], root_dir, check=False)
    
    if result.returncode == 0:
        print_colored("  ✅ Tests passed", Colors.GREEN)
    else:
        print_colored("  ⚠️  Some tests failed", Colors.YELLOW)
        response = input("  Continue anyway? (y/n): ").strip().lower()
        if response not in ('y', 'yes'):
            print_colored("  Aborted.", Colors.RED)
            sys.exit(1)


def git_commit(root_dir: Path, version: str, files: List[str]):
    """Commit version changes"""
    print_step("Committing Changes")
    
    # Add files
    print_colored("  Adding files to git...", Colors.CYAN)
    run_command(["git", "add"] + files, root_dir)
    
    # Commit
    commit_msg = f"chore: bump version to {version}"
    print_colored(f"  Committing: {commit_msg}", Colors.CYAN)
    run_command(["git", "commit", "-m", commit_msg], root_dir)
    
    print_colored("  ✅ Changes committed", Colors.GREEN)


def git_tag(root_dir: Path, version: str, message: str):
    """Create git tag"""
    print_step("Creating Git Tag")
    
    tag_name = f"v{version}"
    print_colored(f"  Creating tag: {tag_name}", Colors.CYAN)
    run_command(["git", "tag", "-a", tag_name, "-m", message], root_dir)
    
    print_colored(f"  ✅ Tag created: {tag_name}", Colors.GREEN)
    return tag_name


def git_push(root_dir: Path, tag_name: str, push_tag: bool = True):
    """Push changes and tag to remote"""
    print_step("Pushing to Remote")
    
    # Push commits
    print_colored("  Pushing commits...", Colors.CYAN)
    run_command(["git", "push", "origin", "main"], root_dir)
    print_colored("  ✅ Commits pushed", Colors.GREEN)
    
    # Push tag
    if push_tag:
        print_colored(f"  Pushing tag {tag_name}...", Colors.CYAN)
        run_command(["git", "push", "origin", tag_name], root_dir)
        print_colored("  ✅ Tag pushed", Colors.GREEN)


def main():
    parser = argparse.ArgumentParser(
        description="Bump version for Reratui crates",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python bump_version.py patch              # 0.2.0 -> 0.2.1
  python bump_version.py minor              # 0.2.0 -> 0.3.0
  python bump_version.py major              # 0.2.0 -> 1.0.0
  python bump_version.py --version 0.3.0    # Set specific version
  python bump_version.py patch --no-test    # Skip tests
  python bump_version.py minor --no-push    # Don't push to remote
        """
    )
    
    parser.add_argument(
        "bump_type",
        nargs="?",
        choices=["major", "minor", "patch"],
        help="Type of version bump (major, minor, or patch)"
    )
    parser.add_argument(
        "--version",
        help="Set specific version (e.g., 0.3.0)"
    )
    parser.add_argument(
        "--no-test",
        action="store_true",
        help="Skip running tests"
    )
    parser.add_argument(
        "--no-commit",
        action="store_true",
        help="Don't commit changes"
    )
    parser.add_argument(
        "--no-tag",
        action="store_true",
        help="Don't create git tag"
    )
    parser.add_argument(
        "--no-push",
        action="store_true",
        help="Don't push to remote"
    )
    parser.add_argument(
        "--tag-message",
        default="",
        help="Custom tag message"
    )
    
    args = parser.parse_args()
    
    # Validate arguments
    if not args.version and not args.bump_type:
        parser.error("Either bump_type or --version must be specified")
    
    # Get root directory
    root_dir = Path(__file__).parent.parent
    
    try:
        # Get current version
        current_version = get_current_version(root_dir)
        print_colored(f"\n📦 Current version: {current_version}", Colors.BOLD + Colors.CYAN)
        
        # Calculate new version
        if args.version:
            new_version = args.version
        else:
            new_version = bump_version(current_version, args.bump_type)
        
        print_colored(f"📦 New version: {new_version}", Colors.BOLD + Colors.GREEN)
        
        # Confirm
        print()
        response = input(f"Bump version from {current_version} to {new_version}? (y/n): ").strip().lower()
        if response not in ('y', 'yes'):
            print_colored("Aborted.", Colors.YELLOW)
            sys.exit(0)
        
        # Update versions
        updated_crates = update_crate_versions(root_dir, current_version, new_version)
        update_workspace_version(root_dir, current_version, new_version)
        
        # Verify build
        verify_build(root_dir)
        
        # Run tests
        if not args.no_test:
            run_tests(root_dir)
        
        # Prepare files for commit
        files = ["Cargo.toml"]
        for crate in updated_crates:
            files.append(f"crates/{crate}/Cargo.toml")
        
        # Git operations
        tag_name = None
        if not args.no_commit:
            git_commit(root_dir, new_version, files)
            
            if not args.no_tag:
                tag_message = args.tag_message or f"Release v{new_version}"
                tag_name = git_tag(root_dir, new_version, tag_message)
            
            if not args.no_push:
                git_push(root_dir, tag_name, push_tag=not args.no_tag)
        
        # Success summary
        print_step("✅ Version Bump Complete!")
        print_colored(f"  Version: {current_version} → {new_version}", Colors.GREEN)
        print_colored(f"  Updated crates: {len(updated_crates)}", Colors.GREEN)
        
        if tag_name:
            print_colored(f"  Git tag: {tag_name}", Colors.GREEN)
        
        print()
        print_colored("Next steps:", Colors.CYAN)
        print_colored("  1. Publish crates: cargo publish --package <crate>", Colors.CYAN)
        print_colored("  2. Create GitHub release", Colors.CYAN)
        print()
        
    except KeyboardInterrupt:
        print()
        print_colored("Cancelled by user.", Colors.YELLOW)
        sys.exit(0)
    except Exception as e:
        print()
        print_colored(f"❌ Error: {e}", Colors.RED)
        sys.exit(1)


if __name__ == "__main__":
    main()
