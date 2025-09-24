#!/bin/bash
# Universal post-install script for both RPM and DEB packages

# Determine the installation prefix
# RPM uses RPM_INSTALL_PREFIX, DEB typically uses /usr
if [ -n "${RPM_INSTALL_PREFIX}" ]; then
    PREFIX="${RPM_INSTALL_PREFIX}"
else
    PREFIX="/usr"
fi

SCHEMAS_DIR="${PREFIX}/share/glib-2.0/schemas"
APPLICATIONS_DIR="${PREFIX}/share/applications"
ICON_DIR="${PREFIX}/share/icons/hicolor"

# Function to safely run commands with error handling
run_command() {
    local command="$1"
    local description="$2"
    
    if command -v "${command%% *}" >/dev/null 2>&1; then
        echo "${description}..."
        eval "${command}" 2>/dev/null || true
    else
        echo "Warning: ${command%% *} not found, skipping ${description,,}"
    fi
}

# Check if schemas directory exists and compile
if [ -d "${SCHEMAS_DIR}" ]; then
    run_command "glib-compile-schemas '${SCHEMAS_DIR}'" "Compiling GLib schemas"
fi

# Update desktop database if applications directory exists
if [ -d "${APPLICATIONS_DIR}" ]; then
    run_command "update-desktop-database '${APPLICATIONS_DIR}'" "Updating desktop database"
fi

# Update icon cache if icon directory exists
if [ -d "${ICON_DIR}" ]; then
    run_command "gtk-update-icon-cache '${ICON_DIR}'" "Updating icon cache"
fi

exit 0