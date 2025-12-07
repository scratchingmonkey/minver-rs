# **Feasibility and Architectural Specification: Porting MinVer to Rust**

## **1. Executive Summary**

The software development industry has increasingly coalesced around Semantic Versioning (SemVer) as the lingua franca of release management. Within the.NET ecosystem, **MinVer** has established a distinct philosophy: versioning should be derived exclusively from Git tags, decoupling the release identifier from build artifacts or configuration files. This "tag-driven" approach simplifies the release process but currently tethers the utility to the.NET SDK. This report presents a comprehensive research analysis and architectural blueprint for re-implementing MinVer in **Rust**.

The primary motivation for this port is to create a zero-dependency, high-performance command-line interface (CLI) that extends MinVer’s utility beyond the.NET ecosystem to languages like Go, Rust, Python, and C++. A Rust-based implementation offers significant advantages in startup latency, binary portability, and memory safety.

Our research indicates that the **`gitoxide` (`gix`)** library, a pure Rust implementation of Git, is the optimal candidate for the underlying engine, superseding the traditional `libgit2` bindings due to its ease of cross-compilation and granular control over graph traversal. The report details the algorithmic reconstruction of MinVer’s "height" calculation—a nuanced metric distinct from standard `git describe`—and provides a complete roadmap for handling repository "dirty" states, traversing divergent merge histories, and automating cross-platform releases via GitHub Actions and Homebrew.

## **2. The Philosophy of Minimalist Versioning**

### **2.1 The Problem with Release Management**

Software versioning has historically been a source of significant friction in Continuous Integration/Continuous Deployment (CI/CD) pipelines. Traditional approaches often fall into two problematic categories: static file management and complex inference engines.

Static file management relies on a `VERSION` or `package.json` file committed to the repository. This method, while simple, introduces a "commit tax" for every release. To release version `1.0.1`, a developer must modify a file, commit the change, and then tag the repository. This redundancy often leads to synchronization errors where the tag (`v1.0.1`) does not match the file content (`1.0.0`), causing confusion in downstream dependency resolution[^1]

Complex inference engines, exemplified by tools like `GitVersion`, attempt to derive version numbers from branch names (e.g., `feature/login` implies a minor bump) and commit messages. While powerful, these tools often require extensive configuration (YAML files) and impose strict branching strategies (`GitFlow`) that may not align with modern trunk-based development practices.2 They also tend to be computationally expensive, scanning the entire graph to calculate a version number, which slows down local development loops.

### **2.2 The MinVer Approach**

MinVer rejects both static files and complex inference. Its philosophy is grounded in the "Unix Philosophy" of doing one thing well. It posits that the **Git tag is the single source of truth**.

The core tenets of the MinVer protocol are:

*   **Truth in Tags:** If a commit is tagged `1.2.3`, the version is `1.2.3`. No file inside the repository can override this[^2]
*   **Height as a Deterministic Metric:** If a commit is not tagged, the version is derived from the *nearest* ancestor tag plus the "height"—the number of commits between the current state and that tag[^3]
*   **Unopinionated Branching:** MinVer does not care if a user is on `main`, `develop`, or `feature-x`. It only cares about the ancestry graph[^4]
*   **Release Candidates are distinct:** The calculation logic changes fundamentally depending on whether the base tag is a stable release (`1.0.0`) or a pre-release (`1.0.0-rc.1`)[^4]

This approach aligns perfectly with Continuous Delivery, where every commit is potentially deployable. By automating the version generation based on the immutable Git history, teams eliminate the human error associated with manual version bumps.

### **2.3 The Case for Rust**

The original MinVer is implemented in C# and distributed as a NuGet package. While efficient for.NET developers, this imposes a runtime dependency on the.NET SDK. A Rust port addresses this limitation through:

*   **Portability:** Rust compiles to static binaries. A Linux `musl` build runs on Alpine, Debian, and RHEL without external dependencies, making it ideal for minimal container environments[^5]
*   **Performance:** Rust’s zero-cost abstractions allow for millisecond-level execution times. For a tool potentially integrated into a shell prompt (to show the current version), this latency reduction is critical[^6]
*   **Ecosystem Neutrality:** A native binary can be used by Node.js, Python, or Go projects, effectively universalizing the MinVer methodology.

## **3. Protocol Specification and Algorithmic Analysis**

To faithfully re-implement MinVer, one must treat its behavior as a formal protocol. The logic is not merely "finding a tag"; it involves a specific traversal heuristic that handles merge commits and divergent history differently from standard Git tools.

### **3.1 The Height Calculation Algorithm**

The concept of "height" is the defining characteristic of MinVer. While `git describe` provides a similar metric, MinVer’s calculation has specific rules regarding merge traversal that a Rust implementation must replicate exactly.

#### **3.1.1 Linear History**

In a linear history, height is simply the integer difference between the current commit index and the tagged commit index.

*   **Scenario:** Tag `1.0.0` is at Commit A. Commit B is a child of A. Commit C is a child of B (HEAD).
*   **Calculation:** The path is C -> B -> A. The height is 2.
*   **Result:** The version is `1.0.1-alpha.0.2` (assuming default auto-increment)[^3]

#### **3.1.2 Divergent History (Merge Commits)**

The complexity arises when history splits and converges. A merge commit has multiple parents. The standard `git log` traversal might interleave commits from different branches based on timestamps. MinVer, however, adheres to a strict parent-order traversal.

MinVer traverses the first parent first. The "first parent" in Git represents the branch that was checked out when the merge occurred (the destination branch). This is typically the "main line" of history.

*   **Rule:** If history diverges, MinVer follows the parents in the order they are stored in the commit object.
*   **Convergence:** If history diverges and then converges (e.g., a feature branch merged into main, which was branched from main), MinVer uses the height on the **first path followed** where the history diverges[^4]

This is a subtle but critical distinction. A standard Breadth-First Search (BFS) might find a "closer" tag on a feature branch that was merged in. However, if MinVer prioritizes the first parent, it maintains the versioning lineage of the main branch. The Rust implementation cannot simply use `git2`'s default topological sort; it must configure the walker to respect this parentage priority or implement a custom iterator.

### **3.2 Semantic Versioning Synthesis**

Once the base tag and height are determined, the version synthesis logic is applied. This logic is state-dependent based on whether the found tag is a Pre-Release or RTM (Release To Manufacturing).

**Table 1: MinVer Version Synthesis Logic**

| Base Tag Type | Example Base | Height | Logic Applied | Resulting Version |
| :---- | :---- | :---- | :---- | :---- |
| **Exact Match** | 1.0.0 | 0 | Use tag exactly. | 1.0.0 |
| **Pre-Release** | 1.0.0-beta.1 | 5 | Append height to pre-release identifiers. | 1.0.0-beta.1.5 |
| **RTM (Stable)** | 1.0.0 | 5 | Increment Patch (default) + Add Default Pre-Release + Add Height. | 1.0.1-alpha.0.5 |
| **No Tag** | N/A | 10 | Assume 0.0.0 base + Add Default Pre-Release + Add Height. | 0.0.0-alpha.0.10 |

**Deep Insight:** The distinction in the RTM case (row 3) is designed to protect semantic stability. If the last release was 1.0.0, any new commits on top of it are *by definition* work towards the *next* version. Since we don't know if the next version is a patch, minor, or major, MinVer defaults to the safest assumption (Patch) and marks it as an alpha.4 This "defensive versioning" ensures that developers don't accidentally release a stable version 1.0.0 again just because they added a commit.

### **3.3 The "Dirty" State**

A repository is considered "dirty" if there are uncommitted changes to tracked files.

*   **Mechanism:** This requires comparing the Working Tree to the Index (Staging Area), and the Index to the HEAD commit.
*   **MinVer Behavior:** MinVer calculates the version based on the *commit*. If the workspace is dirty, the version technically hasn't changed (because the commit hasn't changed). However, build systems often need to know this.
*   **Implementation Requirement:** The Rust tool must provide a mechanism to detect this state, potentially exposing it via metadata (e.g., `1.0.1-alpha.0.5+dirty`), although the core MinVer specification does not mandate this[^8]

### **3.4 Configuration Precedence**

MinVer allows configuration via environment variables and CLI arguments. The Rust implementation must adhere to a strict precedence order:

1.  **CLI Argument:** (e.g., `--minimum-major-minor 2.0`)
2.  **Environment Variable:** (e.g., `MINVERMINIMUMMAJORMINOR=2.0`)
3.  **Default Value:** (e.g., `1.0` if not specified)

This hierarchy allows CI pipelines to set global defaults via environment variables while allowing specific build steps to override them[^4]

## **4. The Rust Ecosystem: Tooling Selection**

The success of this port depends entirely on the library used to interact with Git. The Rust ecosystem presents two primary candidates: `git2` and `gitoxide`.

### **4.1 Candidate A: `git2` (`libgit2` bindings)**

`git2` provides safe Rust bindings to the C library `libgit2`.

*   **Pros:** It is the industry standard, used by Cargo itself. It is extremely feature-complete and battle-tested.
*   **Cons:** It requires compiling C code. This creates a significant barrier for cross-compilation. To build a static binary for Alpine Linux (`x86_64-unknown-linux-musl`) on a macOS machine, one must install a C cross-compiler (`musl-gcc`) and ensure OpenSSL is statically linked. This complexity contradicts the goal of a "simple, portable tool"[^9]

### **4.2 Candidate B: `gitoxide` (`gix`)**

`gitoxide` is a pure Rust implementation of Git, led by Stefan Lankes (Byron).

*   **Pros:** It compiles with the standard Rust toolchain. Cross-compilation is trivial (`cargo build --target...`). It is designed for high performance, often outperforming `libgit2` in traversal operations due to parallelization[^6]
*   **Cons:** The API is newer and subject to change (unstable). It does not yet support every Git feature (like complex merge conflict resolution), but it fully supports the read-only operations MinVer needs (traversal, tag reading, status checks)[^8]

### **4.3 Strategic Decision**

`gitoxide` is the superior choice for this project.  
MinVer is a read-only tool. It does not need to write commits or merge branches. `gitoxide`'s traversal capabilities (`gix-traverse`) are specifically optimized for the kind of graph walking MinVer performs. Furthermore, the ease of producing static binaries with `gitoxide` aligns perfectly with the distribution goals of a modern CLI tool. The ability to verify "dirty" states and read tags is fully implemented in the current version of `gitoxide`[^10]

## **5. Architectural Blueprint**

This section outlines the software architecture for the Rust application, tentatively named `minver-cli`.

### **5.1 Crate Dependency Graph**

To ensure robustness and maintainability, the project will leverage the following crates:

**Table 2: Proposed Dependency Stack**

| Crate | Version | Purpose | Justification |
| :---- | :---- | :---- | :---- |
| **`gix`** | 0.28+ | Core Git Engine | Pure Rust, high-perf graph traversal, status checking. |
| **`clap`** | 4.4+ | CLI Framework | Industry standard. Native support for Environment Variable mapping (`env` attribute). |
| **`semver`** | 1.0 | Version Parsing | Strict adherence to SemVer 2.0.0 spec. Robust sorting and incrementing logic. |
| **`thiserror`** | 1.0 | Library Errors | Defines structured, typed errors for the internal library logic. |
| **`anyhow`** | 1.0 | Application Errors | User-facing error reporting in the main binary. |
| **`tracing`** | 0.1 | Diagnostics | MinVer has a verbosity flag. `tracing` allows structured, leveled logs (`debug`, `info`, `trace`). |
| **`tracing-subscriber`** | 0.3 | Log Output | Formats the `tracing` events for the console. |

### **5.2 Module Structure**

The application should be structured as a workspace or a single crate with a clear separation between the library logic and the binary entry point.

```
minver-cli/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point: Parses CLI args, initializes logger, calls library.
│   ├── lib.rs            # The public API.
│   ├── config.rs         # Defines the Config struct and clap integration.
│   ├── core/
│   │   ├── mod.rs
│   │   ├── walker.rs     # Encapsulates gix interaction and graph traversal.
│   │   ├── calculator.rs # The business logic (Version synthesis).
│   │   └── tags.rs       # Tag parsing and filtering logic.
│   └── output/
│       ├── mod.rs
│       └── env_formatter.rs # Formats output for CI (GITHUB_ENV, etc).
```

### **5.3 Error Handling Strategy**

The .NET runtime handles exceptions globally. In Rust, we must be explicit.

*   **Domain Errors:** Defined in `core/mod.rs` using `thiserror`.
    *   `NoGitRepo`: Directory is not a git repository.
    *   `InvalidTag`: A tag was found but could not be parsed as SemVer.
    *   `DivergentHistory`: (Internal) The graph is in an unexpected state.
*   **User Feedback:** The `main.rs` will use `anyhow` to print friendly error messages. For example, if no repo is found, instead of a panic, it prints: *"Error: No Git repository found in current directory. Run `git init` or change directories."*

## **6. Detailed Implementation Specification**

### **6.1 Initialization and Discovery**

The tool must first locate the `.git` directory. `gix` provides `gix::discover(path)` which safely walks up the directory tree.

*   **Security:** `gix` implements recent Git security changes (e.g., `safe.directory`). The implementation must handle errors where the user does not own the repository directory[^8]

### **6.2 The Traversal Loop (The "Walker")**

This is the most critical component. The goal is to find the nearest ancestor with a valid tag.

**Algorithm:**

1.  **Load Tags:** Retrieve all references from `refs/tags/*`. Filter them by the configured `tag_prefix` (e.g., "v"). Parse valid ones into a `HashMap<ObjectId, Vec<Version>>`.
    *   *Note:* A single commit can have multiple tags. We must store all of them and sort to find the highest precedence version.
2.  **Initialize Walk:** Create a `gix::rev_walk` starting at `HEAD`.
    *   **Sorting:** Use `gix::traverse::commit::Sorting::ByCommitTimeNewestFirst`. This generally aligns with MinVer, though we must verify the "first parent" behavior. `gix` allows customizing the parent traversal order. To strictly match MinVer, we may need to iterate manually over parents if `gix`'s default topological sort deviates from "first-parent" preference in complex merge scenarios[^11]
3.  **Iterate:**
    *   Check current Commit ID against the Tag Map.
    *   If Match: Return (`Version`, `Height`).
    *   If No Match: Increment `Height`. Continue to next parent.
4.  **Termination:** If the walk reaches the root without a tag, return (`Version::new(0,0,0)`, `TotalHeight`).

### **6.3 Handling Shallow Clones**

A major "gotcha" in CI environments is the Shallow Clone (default in GitHub Actions `actions/checkout@v3`). A shallow clone has a truncated history (`depth=1`).

*   **Scenario:** The repo has 100 commits. Tag `v1.0.0` is at commit 90. GitHub checks out commit 100 with `depth=1`.
*   **Failure Mode:** The walker sees commit 100, sees it has no parents (due to shallowness), and assumes it is the root. It returns `0.0.0-alpha.0.0`.
*   **Detection:** `gix` exposes `repo.is_shallow()`.
*   **Mitigation:** The Rust tool **must** check `repo.is_shallow()`. If true, and no tag is found immediately, it should emit a warning to stderr: *"Warning: Shallow repository detected. Version calculation may be incorrect. Fetch full history with `git fetch --unshallow`."*[^12]

### **6.4 Status Checking (Is Dirty?)**

MinVer conceptually runs on the commit. However, to support `MINVERBUILDMETADATA`, we often want to append `dirty` if files are modified.

*   **Implementation:** Use `repo.index_or_empty()` and `repo.status()`.
*   **Code Snippet Logic:**
    ```rust
    let status = repo.status(gix::progress::Discard)?;
    // We only care if tracked files are modified, not untracked files
    let is_dirty = status.into_iter().any(|item| {
        matches!(item.status, gix::status::Status::Modified | gix::status::Status::Added | gix::status::Status::Deleted)
    });
    ```

*   This check is heavily optimized in `gix` and should add negligible overhead[^8]

### **6.5 Configuration Mapping**

MinVer has a specific set of options that must be mapped to `clap`.

**Table 3: Configuration Mapping**

| MinVer Option | Env Variable | CLI Flag | Rust Struct Field |
| :---- | :---- | :---- | :---- |
| **`AutoIncrement`** | `MINVERAUTOINCREMENT` | `-a`, `--auto-increment` | `auto_increment: Option<String>` |
| **`BuildMetadata`** | `MINVERBUILDMETADATA` | `-b`, `--build-metadata` | `build_metadata: Option<String>` |
| **`DefaultPreReleasePhase`** | `MINVERDEFAULTPRERELEASEPHASE` | `-d`, `--default-pre-release-phase` | `default_pre_release_phase: String` |
| **`MinimumMajorMinor`** | `MINVERMINIMUMMAJORMINOR` | `-m`, `--minimum-major-minor` | `minimum_major_minor: Option<String>` |
| **`TagPrefix`** | `MINVERTAGPREFIX` | `-t`, `--tag-prefix` | `tag_prefix: String` |
| **`Verbosity`** | `MINVERVERBOSITY` | `-v`, `--verbosity` | `verbosity: String` |
| **`IgnoreHeight`** | `MINVERIGNOREHEIGHT` | `-i`, `--ignore-height` | `ignore_height: bool` |

*Insight:* The `IgnoreHeight` option is interesting. If set, MinVer calculates the version as if the height were 0 (conceptually). This is useful for local builds where you want to generate the "next" version number cleanly without the `.0.42` suffix[^4]

## **7. Operational Strategy: Release and Distribution**

The primary advantage of the Rust port is the ease of distribution. We can distribute a single binary, `minver`, that has no dependencies.

### **7.1 Cross-Compilation Matrix**

We must target the major platforms used in CI pipelines.

*   **Linux (x64):** `x86_64-unknown-linux-musl`. (Static binary).
*   **Linux (ARM64):** `aarch64-unknown-linux-musl`. (For AWS Graviton / Raspberry Pi CI runners).
*   **macOS (Intel):** `x86_64-apple-darwin`.
*   **macOS (Apple Silicon):** `aarch64-apple-darwin`.
*   **Windows:** `x86_64-pc-windows-msvc`.

### **7.2 GitHub Actions Release Workflow**

We can automate the entire release process using GitHub Actions.

**Workflow Steps:**

1.  **Trigger:** On push to tag `v*`.
2.  **Job (Matrix):** Run on `ubuntu-latest`, `macos-latest`, `windows-latest`.
3.  **Build:**
    *   Use `cross` (a Rust tool that uses Docker) to build the `musl` targets on the Ubuntu runner.
    *   Standard `cargo build --release` for Mac/Windows.
4.  **Package:** Tar/Zip the binary.
5.  **Release:** Use `softprops/action-gh-release` to upload artifacts to the GitHub Release[^5]

### **7.3 Homebrew Tap Automation**

To facilitate adoption, we must support `brew install minver`.

*   **Repository:** Create `github.com/my-org/homebrew-tap`.
*   **Formula Generation:** We can use a custom script or action (like `formulaic` or `homebrew-releaser`) in the release workflow.
*   **Logic:**
    1.  Calculate SHA256 of the `apple-darwin` tarball.
    2.  Update the `minver.rb` file in the Tap repo with the new version and SHA.
    3.  Commit and push.
*   **Result:** Users run `brew tap my-org/tap` then `brew install minver`[^13]

## **8. Comparative Analysis: MinVer vs. Competitors**

To justify the effort of re-implementation, we must contextualize MinVer against its peers.

### **8.1 MinVer vs. `GitVersion`**

**`GitVersion`** is the "heavyweight" champion.

*   **Logic:** It infers versions from branch names (GitFlow support), merge messages, and tags.
*   **Configuration:** Requires complex `GitVersion.yml`.
*   **Performance:** Can be very slow on large repositories because it often walks the entire graph to determine the "base" version context.
*   **MinVer Advantage:** MinVer is linear O(N) where N is the distance to the nearest tag. It ignores branch names. It is significantly faster and more predictable.

### **8.2 MinVer vs. `Nerdbank.GitVersioning` (NBGV)**

**NBGV** relies on a `version.json` file in the repo to set the Major/Minor version, using Git height for the Patch.

*   **Conflict:** This creates a hybrid model (File + Git).
*   **MinVer Advantage:** MinVer is pure Git. No file editing is required to bump a version (just tag).

### **8.3 Rust Implementation vs..NET Implementation**

*   **Startup Time:** The.NET tool typically takes 200ms+ to start (JIT compilation). A Rust binary will start in < 10ms. This makes it viable for shell prompt integration (showing the version in `PS1`).
*   **Deployment:** The.NET tool requires the.NET SDK or a large (60MB+) self-contained executable. The Rust binary will be \~2-4MB and require nothing.

## **9. Migration Guide for Existing Users**

For users currently using the `MinVer.NET` package, the transition to the Rust CLI should be seamless.

### **9.1 Compatibility Layer**

The Rust tool should support the exact same environment variables (`MINVER*`). This allows teams to swap the tool in their pipeline without rewriting their shell scripts or YAML definitions.

### **9.2 Build Integration**

Currently, MinVer integrates into MSBuild:

```xml
<PackageReference Include="MinVer" Version="4.3.0" />
```

To use the Rust tool, users would modify their project file to execute the CLI:

```xml
<PropertyGroup>
  <MinVerToolPath>minver</MinVerToolPath>
</PropertyGroup>
<Target Name="SetVersion" BeforeTargets="BeforeBuild">
  <Exec Command="$(MinVerToolPath) --auto-increment patch" ConsoleToMSBuild="true">
    <Output TaskParameter="ConsoleOutput" PropertyName="Version" />
  </Exec>
</Target>
```

While slightly more verbose, this decouples the build from the NuGet package restore cycle.

## **10. Conclusion and Future Outlook**

Re-implementing MinVer in Rust is not merely a translation exercise; it is an architectural optimization. By leveraging **gitoxide**, we gain access to state-of-the-art graph traversal performance and memory safety. The resulting tool fills a critical gap in the non-.NET ecosystem for a simple, unopinionated, tag-driven versioning utility.

The research confirms that all core requirements—height calculation, dirty state detection, and configuration mapping—are fully supportable by the current Rust crate ecosystem. The proposed distribution strategy via static binaries and Homebrew Taps ensures that this tool can become a standard utility in the DevOps toolkit, independent of the language or framework of the project being versioned.

This blueprint provides the necessary technical depth for a systems engineer to begin implementation immediately, with clear guidance on the most complex aspects of the protocol: handling merge commits and detecting shallow clones. The future of versioning is deterministic, and with this tool, it is also universally accessible.

#### **Works cited**

[^1]: The Easiest Way to Version NuGet Packages - Muhammad Rehan Saeed, accessed December 6, 2025, [https://rehansaeed.com/the-easiest-way-to-version-nuget-packages/](https://rehansaeed.com/the-easiest-way-to-version-nuget-packages/)  
[^2]: minver/README.md at main - GitHub, accessed December 6, 2025, [https://github.com/adamralph/minver/blob/main/README.md](https://github.com/adamralph/minver/blob/main/README.md)  
[^3]: Simplifying NuGet Versioning with MinVer - Rody van Sambeek, accessed December 6, 2025, [https://www.rodyvansambeek.com/blog/simplifying-nuget-versioning-with-minver](https://www.rodyvansambeek.com/blog/simplifying-nuget-versioning-with-minver)  
[^4]: adamralph/minver: Minimalistic versioning using Git tags. - GitHub, accessed December 6, 2025, [https://github.com/adamralph/minver](https://github.com/adamralph/minver)  
[^5]: How to Deploy Rust Binaries with GitHub Actions - dzfrias, accessed December 6, 2025, [https://dzfrias.dev/blog/deploy-rust-cross-platform-github-actions/](https://dzfrias.dev/blog/deploy-rust-cross-platform-github-actions/)  
[^6]: Playing around with gitoxide - an implementation of git in Rust - twdev.blog, accessed December 6, 2025, [https://twdev.blog/2023/12/gitoxide/](https://twdev.blog/2023/12/gitoxide/)  
[^7]: MinVer 3.0.0 - NuGet, accessed December 6, 2025, [https://www.nuget.org/packages/MinVer/3.0.0](https://www.nuget.org/packages/MinVer/3.0.0)  
[^8]: Repository in gix - Rust - Docs.rs, accessed December 6, 2025, [https://docs.rs/gix/latest/gix/struct.Repository.html](https://docs.rs/gix/latest/gix/struct.Repository.html)  
[^9]: gitoxide integration in cargo. First PR merged. Now it's possible to use gitoxide instead of libgit2 to fetch git repos. : r/rust - Reddit, accessed December 6, 2025, [https://www.reddit.com/r/rust/comments/11g3i7t/gitoxide_integration_in_cargo_first_pr_merged_now/](https://www.reddit.com/r/rust/comments/11g3i7t/gitoxide_integration_in_cargo_first_pr_merged_now/)  
[^10]: GitoxideLabs/gitoxide: An idiomatic, lean, fast & safe pure Rust implementation of Git - GitHub, accessed December 6, 2025, [https://github.com/GitoxideLabs/gitoxide](https://github.com/GitoxideLabs/gitoxide)  
[^11]: Commit ancestor traversal sorted by commit time · Issue #270 · GitoxideLabs/gitoxide, accessed December 6, 2025, [https://github.com/Byron/gitoxide/issues/270](https://github.com/Byron/gitoxide/issues/270)  
[^12]: Tracking Issue for -Zgitoxide #11813 - rust-lang/cargo - GitHub, accessed December 6, 2025, [https://github.com/rust-lang/cargo/issues/11813](https://github.com/rust-lang/cargo/issues/11813)  
[^13]: How to Publish your Rust project on Homebrew - Federico Terzi, accessed December 6, 2025, [https://federicoterzi.com/blog/how-to-publish-your-rust-project-on-homebrew/](https://federicoterzi.com/blog/how-to-publish-your-rust-project-on-homebrew/)  
[^14]: Homebrew Releaser · Actions · GitHub Marketplace, accessed December 6, 2025, [https://github.com/marketplace/actions/homebrew-releaser](https://github.com/marketplace/actions/homebrew-releaser)