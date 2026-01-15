/**
 * CI/CD Configuration Tests for pack-league
 *
 * These tests verify that the release workflow will succeed by checking:
 * 1. Cargo.toml has git URL (not path) for companion-pack-protocol
 * 2. package.json has correct dependency format for CI
 * 3. Required workflow files exist
 * 4. GitHub secrets documentation is up to date
 *
 * Run with: pnpm test (or click the play button in VS Code)
 */

import { describe, it, expect } from "vitest";
import { readFileSync, existsSync } from "fs";
import { join } from "path";

const PACK_ROOT = join(__dirname, "..");
const DAEMON_ROOT = join(PACK_ROOT, "daemon");

describe("CI/CD Configuration", () => {
  describe("Rust daemon dependencies", () => {
    it("has Cargo.toml", () => {
      const cargoPath = join(DAEMON_ROOT, "Cargo.toml");
      expect(existsSync(cargoPath)).toBe(true);
    });

    it("uses git URL for companion-pack-protocol (not local path)", () => {
      const cargoPath = join(DAEMON_ROOT, "Cargo.toml");
      const content = readFileSync(cargoPath, "utf-8");

      // Check that we're using git URL, not path
      const hasGitUrl = content.includes(
        'companion-pack-protocol = { git = "https://github.com/clip-companion/companion-pack-protocol'
      );
      const hasLocalPath = content.includes(
        'companion-pack-protocol = { path = '
      );

      // For CI to work, we need git URL. Local path is only for development.
      // If this test fails, uncomment the git URL line and comment out the path line in Cargo.toml
      expect(hasGitUrl).toBe(true);
      expect(hasLocalPath).toBe(false);
    });

    it("git URL points to clip-companion org (not personal account)", () => {
      const cargoPath = join(DAEMON_ROOT, "Cargo.toml");
      const content = readFileSync(cargoPath, "utf-8");

      // Ensure we're pointing to the org, not a personal account
      expect(content).toContain("github.com/clip-companion/");
      expect(content).not.toMatch(/github\.com\/[^c][^l][^i][^p].*\/companion-pack-protocol/);
    });
  });

  describe("Frontend dependencies", () => {
    it("has package.json", () => {
      const pkgPath = join(PACK_ROOT, "frontend", "package.json");
      expect(existsSync(pkgPath)).toBe(true);
    });

    it("uses git URL for @companion/gamepack-runtime (not local path)", () => {
      const pkgPath = join(PACK_ROOT, "frontend", "package.json");
      const content = readFileSync(pkgPath, "utf-8");
      const pkg = JSON.parse(content);

      const protocolDep = pkg.dependencies?.["@companion/gamepack-runtime"];

      // For CI to work, we need a git URL or npm package, not file: path
      // If this test fails, change the dependency to use git URL
      if (protocolDep) {
        expect(protocolDep).not.toMatch(/^file:/);
        // Should be either a git URL or semver version
        const isGitUrl = protocolDep.includes("github.com");
        const isSemver = /^\d+\.\d+\.\d+/.test(protocolDep) || protocolDep.startsWith("^") || protocolDep.startsWith("~");
        const isGitProtocol = protocolDep.startsWith("git+") || protocolDep.startsWith("github:");
        expect(isGitUrl || isSemver || isGitProtocol).toBe(true);
      }
    });
  });

  describe("GitHub Actions workflow", () => {
    it("has release.yml workflow", () => {
      const workflowPath = join(PACK_ROOT, ".github", "workflows", "release.yml");
      expect(existsSync(workflowPath)).toBe(true);
    });

    it("workflow triggers on push to main", () => {
      const workflowPath = join(PACK_ROOT, ".github", "workflows", "release.yml");
      const content = readFileSync(workflowPath, "utf-8");

      expect(content).toContain("push:");
      expect(content).toContain("branches:");
      expect(content).toContain("main");
    });

    it("workflow has version bump job", () => {
      const workflowPath = join(PACK_ROOT, ".github", "workflows", "release.yml");
      const content = readFileSync(workflowPath, "utf-8");

      expect(content).toContain("version:");
      expect(content).toContain("Bump version");
    });

    it("workflow has daemon build job for all platforms", () => {
      const workflowPath = join(PACK_ROOT, ".github", "workflows", "release.yml");
      const content = readFileSync(workflowPath, "utf-8");

      // Check all platform targets are present
      expect(content).toContain("aarch64-apple-darwin"); // macOS ARM
      expect(content).toContain("x86_64-apple-darwin"); // macOS Intel
      expect(content).toContain("x86_64-unknown-linux-gnu"); // Linux
      expect(content).toContain("x86_64-pc-windows-msvc"); // Windows
    });

    it("workflow has frontend build job", () => {
      const workflowPath = join(PACK_ROOT, ".github", "workflows", "release.yml");
      const content = readFileSync(workflowPath, "utf-8");

      expect(content).toContain("build-frontend:");
      expect(content).toContain("pnpm build");
    });

    it("workflow creates GitHub release", () => {
      const workflowPath = join(PACK_ROOT, ".github", "workflows", "release.yml");
      const content = readFileSync(workflowPath, "utf-8");

      expect(content).toContain("release:");
      expect(content).toContain("softprops/action-gh-release");
    });

    it("workflow updates packs-index via webhook", () => {
      const workflowPath = join(PACK_ROOT, ".github", "workflows", "release.yml");
      const content = readFileSync(workflowPath, "utf-8");

      expect(content).toContain("Update packs-index");
      expect(content).toContain("PACKS_INDEX_TOKEN");
      expect(content).toContain("clip-companion/packs-index");
      // Uses repository_dispatch via curl to /dispatches endpoint
      expect(content).toContain("/dispatches");
      expect(content).toContain("pack-released");
    });
  });

  describe("Config files", () => {
    it("has config.json with version", () => {
      const configPath = join(PACK_ROOT, "config.json");
      expect(existsSync(configPath)).toBe(true);

      const content = readFileSync(configPath, "utf-8");
      const config = JSON.parse(content);

      expect(config.version).toBeDefined();
      expect(config.version).toMatch(/^\d+\.\d+\.\d+$/);
    });

    it("has valid pack_id (UUID format)", () => {
      const configPath = join(PACK_ROOT, "config.json");
      const content = readFileSync(configPath, "utf-8");
      const config = JSON.parse(content);

      expect(config.pack_id).toBeDefined();
      // UUID format: 8-4-4-4-12 hex characters
      expect(config.pack_id).toMatch(
        /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i
      );
    });
  });
});

describe("Required Secrets Documentation", () => {
  it("README documents PACKS_INDEX_TOKEN setup", () => {
    const readmePath = join(PACK_ROOT, "README.md");
    if (existsSync(readmePath)) {
      const content = readFileSync(readmePath, "utf-8");
      // The README should mention the token somewhere
      const mentionsToken =
        content.includes("PACKS_INDEX_TOKEN") ||
        content.includes("personal access token") ||
        content.includes("PAT");
      expect(mentionsToken).toBe(true);
    }
  });
});
