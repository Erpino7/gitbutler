# Container for running E2E tests on a mac.
#
# 1. docker build --tag gitbutler/e2e
# 2. docker run --name e2e-agent --detach -p 2222:22 -v $GITBUTLER_REPO:/root/gitbutler \
#     -v /tmp/e2e-pnpm-store:/tmp/pnpm-store gitbutler/e2e
# 3. docker cp -a ~/.ssh/id_ed25519.pub e2e-agent:/root/.ssh/authorized_keys
# 4. ssh -p 2222 -Y root@localhost
# 5. cd /root/gitbutler
# 6. pnpm install && cargo build 
# 7. pnpm build:test
# 8. pnpm test:e2e

FROM ubuntu:24.04

RUN apt-get update

RUN apt-get install -y \
    build-essential \
    curl \
    git \
    pkg-config \
    psmisc \
    vim

# Install Tauri dependencies
# https://tauri.app/v1/guides/getting-started/prerequisites#setting-up-linux
RUN apt-get install -y \
    webkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# Install tauri-driver dependencies
RUN apt-get install -y \
    dbus-x11 \
    webkit2gtk-driver \
    xvfb

# Install ssh server for X11 forwarding
RUN apt-get install -y openssh-server

# Needed for recording test execution.
RUN apt-get install -y ffmpeg

# Install gitbutler dependencies
RUN apt-get install -y cmake

# Needed by at least pnpm.
ENV SHELL=bash

# Install rust.
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install node.
RUN curl -fsSL "https://deb.nodesource.com/setup_20.x" | bash -
RUN apt-get install -y nodejs
RUN corepack enable

# Install pnpm and configure store directory.
ENV PNPM_HOME=/tmp/.pnpm
RUN wget -qO- https://get.pnpm.io/install.sh | ENV="$HOME/.bashrc" SHELL="$(which bash)" bash -
# Home/store dir should be mounted from the host.
RUN pnpm config set store-dir /tmp/.pnpm 
ENV PATH="/tmp/.pnpm:${PATH}"
RUN pnpm setup

# Used to manage build dependencies between packages.
RUN pnpm add --global turbo

# Used as a proxy for communicating with WebKitWebDriver.
RUN cargo install tauri-driver

# vi bindings on by default.
RUN echo "set -o vi" >> /root/.bashrc

# Clean up before finishing up.
RUN apt-get -y autoremove && apt-get -y clean

# Run SSH server in foreground.
CMD ["service", "ssh", "start", "-D"]


