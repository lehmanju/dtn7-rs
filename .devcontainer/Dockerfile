# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.224.3/containers/rust/.devcontainer/base.Dockerfile

# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="buster"
FROM mcr.microsoft.com/vscode/devcontainers/rust:0-${VARIANT}

# [Optional] Uncomment this section to install additional packages.
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends x11-xserver-utils ca-certificates \
    curl \
    gnupg \
    lsb-release \
    libtk-img \
    tk

RUN curl -fsSL https://download.docker.com/linux/debian/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
RUN echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/debian \
  $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

RUN groupadd docker

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends docker-ce docker-ce-cli containerd.io

VOLUME [ "/var/lib/docker" ]

ENTRYPOINT sudo rm -f /var/run/docker.pid && sudo dockerd -H unix:///var/run/docker.sock
  
