# Base image ships a recent stable Rust toolchain (>= 1.85, required by
# edition 2024) plus git/curl/build-essential via buildpack-deps.
FROM rust:1-bookworm

# System libraries required to *build* and *run* waver headlessly:
#   - pkg-config + libasound2-dev: cpal links against ALSA.
#   - libx11/xcb/xrandr/xi/xkbcommon/wayland-dev: winit windowing backends.
#   - libgl/egl-dev + mesa-dri + mesa-vulkan-drivers: software rendering
#     (llvmpipe GL / lavapipe Vulkan) so the wgpu renderer works without a GPU.
#   - xvfb + x11-utils: virtual framebuffer for GUI runs on a headless VM.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config \
        libasound2-dev \
        libx11-dev \
        libxcb1-dev \
        libxrandr-dev \
        libxi-dev \
        libxkbcommon-dev \
        libwayland-dev \
        libgl1-mesa-dev \
        libegl1-mesa-dev \
        libgl1-mesa-dri \
        mesa-vulkan-drivers \
        xvfb \
        x11-utils \
    && rm -rf /var/lib/apt/lists/*
