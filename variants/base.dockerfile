FROM gitpod/openvscode-server:latest

USER root
RUN sed -i 's/archive.ubuntu.com/mirrors.tuna.tsinghua.edu.cn/' /etc/apt/sources.list
RUN sed -i 's/security.ubuntu.com/mirrors.tuna.tsinghua.edu.cn/' /etc/apt/sources.list
RUN apt update \
  && apt install -y build-essential fish g++ clangd clang-format golang-go nodejs npm default-jdk vim neovim emacs meson ninja-build \
  && apt clean

USER openvscode-server
ENV OPENVSCODE_SERVER_ROOT="/home/.openvscode-server"
ENV OPENVSCODE="${OPENVSCODE_SERVER_ROOT}/bin/openvscode-server"
SHELL ["/bin/bash", "-c"]
RUN exts=(llvm-vs-code-extensions.vscode-clangd eamodio.gitlens ms-python.python ms-toolsai.jupyter) \
  && for ext in "${exts[@]}"; do ${OPENVSCODE} --install-extension "${ext}"; done
