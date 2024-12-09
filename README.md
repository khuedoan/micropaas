# MicroPaaS

Git push to deploy - slightly larger than
[NanoPaas](https://github.com/khuedoan/nanopaas), but with more goodies :)

## Usage

### Container image build

A `Dockerfile` in the project root is automatically built on push for any
branch. If it's the default branch, the image will be pushed to the registry
and deployed automatically via the GitOps repository.

### CI

If a project has a `flake.nix` file and a `Makefile` with a `ci:` target, CI
will automatically run on push for any branch. The `ci` target has access to
the following variables:

| Variable      | Description                                                 | Example value                                |
| ------------- | ----------------------------------------------------------- | -------------------------------------------- |
| `REF_NAME`    | The name of the ref being updated                           | `refs/heads/master`                          |
| `OLD_OBJECT`  | The old object name stored in the ref                       | `4e54684045b5d2cda33b9843fd6de80863cb97ee`   |
| `NEW_OBJECT`  | The new object name to be stored in the ref                 | `2423e33f06df9ef080f931a0e39b41f0287837b1`   |
| `CACHE_DIR`   | Shared cache directory for each ref name on each repository | `/var/lib/cache/micropaas/refs/heads/master` |

Example `Makefile` with a `ci` target:

```make
ci:
	# Create a target cache directory for Cargo build output
	mkdir -p "${CACHE_DIR}/target"
	# Symlink the target cache directory to the build directory
	ln -s "${CACHE_DIR}/target" "target"
	# Run tests utilizing the cached target directory
	cargo test
```

## Configuration

| Environment variable | Default           | Description                                        |
| -------------------- | ----------------- | -------------------------------------------------- |
| `DEFAULT_BRANCH`     | `master`          | Default branch that triggers deployment on push    |
| `DOCKER_HOST`        |                   | Specifies the remote Docker host                   |
| `GITOPS_REPO`        |                   | Specifies the GitOps repository used in deployment |
| `REGISTRY_HOST`      |                   | Hostname or prefix for the container registry      |
| `GIT_USER_NAME`      | `Bot`             | Git committer user name for the deploy step        |
| `GIT_USER_EMAIL`     | `bot@example.com` | Git committer user email for the deploy step       |
