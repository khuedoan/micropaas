# MicroPaaS

Git push to deploy - slightly larger than
[NanoPaas](https://github.com/khuedoan/nanopaas), but with more goodies :)

## Configuration

| Environment variable | Default     | Description                                        |
| -------------------- | ----------- | -------------------------------------------------- |
| `DEFAULT_BRANCH`     | `master`    | Default branch that triggers deployment on push    |
| `DOCKER_HOST`        |             | Specifies the remote Docker host                   |
| `GITOPS_REPO`        | `gitops`    | Specifies the GitOps repository used in deployment |
| `REGISTRY_HOST`      | `docker.io` | Hostname or prefix for the container registry      |
