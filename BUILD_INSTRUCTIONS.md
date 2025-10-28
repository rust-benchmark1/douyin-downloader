## Prerequisites

- Ubuntu 20.04 or later (tested on Ubuntu 24.04 Noble)
- Node.js and npm installed
- Rust toolchain installed (rustc, cargo)

## System Dependencies

### Install Required Libraries

Install the necessary system libraries for Tauri and GTK:

```bash
sudo apt-get update

sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libsoup2.4-dev
```

### Create Compatibility Symlinks

Ubuntu 24.04 only provides webkit2gtk-4.1, but this project requires webkit2gtk-4.0. Create symlinks for compatibility:

#### pkg-config Files

```bash
sudo ln -sf /usr/lib/x86_64-linux-gnu/pkgconfig/javascriptcoregtk-4.1.pc /usr/lib/x86_64-linux-gnu/pkgconfig/javascriptcoregtk-4.0.pc

sudo ln -sf /usr/lib/x86_64-linux-gnu/pkgconfig/webkit2gtk-4.1.pc /usr/lib/x86_64-linux-gnu/pkgconfig/webkit2gtk-4.0.pc

sudo ln -sf /usr/lib/x86_64-linux-gnu/pkgconfig/webkit2gtk-web-extension-4.1.pc /usr/lib/x86_64-linux-gnu/pkgconfig/webkit2gtk-web-extension-4.0.pc
```

#### Shared Libraries

```bash
sudo ln -sf /usr/lib/x86_64-linux-gnu/libwebkit2gtk-4.1.so /usr/lib/x86_64-linux-gnu/libwebkit2gtk-4.0.so

sudo ln -sf /usr/lib/x86_64-linux-gnu/libjavascriptcoregtk-4.1.so /usr/lib/x86_64-linux-gnu/libjavascriptcoregtk-4.0.so
```

## Building the Application

### 1. Clone the Repository

### 2. Install Frontend Dependencies

Install npm packages for the React frontend:

```bash
npm install
```

### 3. Build the Frontend

Build the React application (creates the `build/` directory):

```bash
npm run build
```

### 4. Build the Tauri Application

Navigate to the Tauri directory and build:

```bash
cd src-tauri
cargo build
```

For a release build (optimized):

```bash
cargo build --release
```

## Running the Application

### Development Mode

To run the application in development mode with hot-reload:

```bash
# From the project root directory
npm run tauri dev
```

This will:

1. Start the React development server on `http://localhost:3000`
2. Launch the Tauri application window
3. Enable hot-reload for frontend changes

### Production Build

To create a production-ready application bundle:

```bash
npm run tauri build
```

The compiled application will be located in:

- **Executable**: `src-tauri/target/release/app`
- **Installer packages**: `src-tauri/target/release/bundle/`

## Common Issues

### Build Errors Related to webkit2gtk

If you encounter errors about missing `libwebkit2gtk-4.0` or `javascriptcoregtk-4.0`:

- Verify all symlinks were created correctly
- Check that `libwebkit2gtk-4.1-dev` is installed

### Missing `build/` Directory Error

If you see: `The 'distDir' configuration is set to "../build" but this path doesn't exist`

- Run `npm run build` from the project root before building the Tauri app

### Permission Denied on Symlinks

If symlink creation fails:

- Ensure you're using `sudo` for the symlink commands
- Verify you have write permissions to `/usr/lib/x86_64-linux-gnu/`

## Project Structure

```
douyin-downloader/
├── src/                  # React frontend source
├── public/               # Public assets
├── build/                # Built frontend (created by npm run build)
├── src-tauri/            # Tauri backend
│   ├── src/              # Rust source code
│   ├── target/           # Cargo build output
│   └── Cargo.toml        # Rust dependencies
├── package.json          # Node.js dependencies
└── tauri.conf.json       # Tauri configuration
```

## Quick Start Commands

For a fresh setup, run these commands in order:

```bash
# 1. Install system dependencies
sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libsoup2.4-dev

# 2. Create symlinks (pkg-config)
sudo ln -sf /usr/lib/x86_64-linux-gnu/pkgconfig/javascriptcoregtk-4.1.pc /usr/lib/x86_64-linux-gnu/pkgconfig/javascriptcoregtk-4.0.pc
sudo ln -sf /usr/lib/x86_64-linux-gnu/pkgconfig/webkit2gtk-4.1.pc /usr/lib/x86_64-linux-gnu/pkgconfig/webkit2gtk-4.0.pc
sudo ln -sf /usr/lib/x86_64-linux-gnu/pkgconfig/webkit2gtk-web-extension-4.1.pc /usr/lib/x86_64-linux-gnu/pkgconfig/webkit2gtk-web-extension-4.0.pc

# 3. Create symlinks (libraries)
sudo ln -sf /usr/lib/x86_64-linux-gnu/libwebkit2gtk-4.1.so /usr/lib/x86_64-linux-gnu/libwebkit2gtk-4.0.so
sudo ln -sf /usr/lib/x86_64-linux-gnu/libjavascriptcoregtk-4.1.so /usr/lib/x86_64-linux-gnu/libjavascriptcoregtk-4.0.so

# 4. Install npm dependencies
npm install

# 5. Build frontend
npm run build

# 6. Build Tauri app
cd src-tauri && cargo build

# 7. Run development mode
cd .. && npm run tauri dev
```
