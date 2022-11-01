# kkowa/apps/proxy

Man-in-the-middle proxy component for data collection for kkowa.

## 🧰 Tech Stack

- **Language** Rust 1
- **Framework** [hyper](https://hyper.rs/)
- **Source Control** Git and GitLab
- **CI·CD** GitLab CI

## ⚙️ Getting Started

This section describes how to set your local environments up.

### Setup

Followings are **required**.

- [Docker](https://www.docker.com/)

  To configure other dependent services like database, we use Docker (mainly [**Docker Compose**](https://docs.docker.com/compose/)).

- **(A)** Developing With Development Container

  - [Visual Studio Code](https://code.visualstudio.com/)

    Basically VS Code Development Container provides rich features such as git configuration and GPG sharing. But they sometimes require you to install some tools based on your device. Please check [this](https://code.visualstudio.com/docs/remote/containers#_sharing-git-credentials-with-your-container).

- **(B)** Developing Locally

  - [Rust](https://www.rust-lang.org/)

After you installed all above, then follow next steps based on your choice (A, B):

#### **(A)** Developing With Development Container

We configured all basic tools to be installed inside devcontainer, such as **Pipenv**, etc.

1. Install VS Code extension **Remote - Containers (by Microsoft)**.

1. Then, clone this repository and open in VS Code, select **Remote-Containers: Reopen in Container...** at command palette (<kbd>ctrl</kbd> + <kbd>shift</kbd> + <kbd>P</kbd> or <kbd>cmd</kbd> + <kbd>shift</kbd> + <kbd>P</kbd>).

1. Done. Container includes required tools such as **pre-commit**, so you are ready to code.

#### **(B)** Developing Locally

1. Run `make install`

1. Run `make init`

1. Done. all other configurations are on your own. Or, you can use existing docker compose file to create dependent services (but would require some configuration changes).

#### **(C)** Remote Environment: GitHub Codespace or GitPod

In consideration but not ready to adopt it yet.

### 💯 pre-commit

We are using [pre-commit](https://pre-commit.com/) to check common lint errors and for code formatting. Basically you don't need to anything to install it because it is already listed in project's Pipfile.

What you have to do is just run `pre-commit install` (or `make init`)

### 🐋 Docker Compose

You can see composed environment at [docker-compose.yml](./docker-compose.yml) file. To say shortly, exposed services would be:

- **proxy** at port **8080**

If you are using Docker based on VM (like **Docker ToolBox**), localhost won't work for you. Follow [this](https://stackoverflow.com/a/42886035).

### ⌨️ Basic Commands

Convenience scripts are defined in [Makefile](./Makefile) at project root. `make` without arguments will show you possible commands.
