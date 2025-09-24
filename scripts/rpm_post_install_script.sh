#!/bin/bash
# Post-install script with dynamic path
SCHEMAS_DIR="${RPM_INSTALL_PREFIX}/share/glib-2.0/schemas"

# Check if schemas directory exists and compile
if [ -d "${SCHEMAS_DIR}" ]; then
    glib-compile-schemas "${SCHEMAS_DIR}"
fi