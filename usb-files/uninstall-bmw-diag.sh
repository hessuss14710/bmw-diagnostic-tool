#!/bin/bash
#
# BMW Diagnostic Tool - Script de Desinstalación
#

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[✓]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[✗]${NC} $1"; }

if [ "$EUID" -ne 0 ]; then
    error "Ejecutar como root: sudo $0"
    exit 1
fi

echo ""
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║     BMW Diagnostic Tool - Desinstalación                  ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

read -p "¿Estás seguro de que quieres desinstalar? (s/N): " confirm
if [[ ! "$confirm" =~ ^[sS]$ ]]; then
    echo "Cancelado"
    exit 0
fi

echo ""

# Desinstalar paquete
if dpkg -l | grep -q bmw-diag; then
    log "Desinstalando paquete bmw-diag..."
    apt-get remove -y bmw-diag
fi

# Eliminar AppImage si existe
if [ -f /opt/bmw-diag.AppImage ]; then
    log "Eliminando AppImage..."
    rm -f /opt/bmw-diag.AppImage
fi

# Eliminar utilidades
log "Eliminando utilidades..."
rm -f /usr/local/bin/kdcan-test
rm -f /usr/local/bin/kdcan-monitor

# Eliminar acceso directo
REAL_USER=${SUDO_USER:-$USER}
REAL_HOME=$(eval echo ~$REAL_USER)
rm -f "$REAL_HOME/Escritorio/BMW Diagnostic Tool.desktop" 2>/dev/null
rm -f "$REAL_HOME/Desktop/BMW Diagnostic Tool.desktop" 2>/dev/null

# Preguntar si eliminar datos
echo ""
read -p "¿Eliminar también los datos de la aplicación? (s/N): " delete_data
if [[ "$delete_data" =~ ^[sS]$ ]]; then
    log "Eliminando datos..."
    rm -rf "$REAL_HOME/.local/share/com.bmw-diag.app"
fi

# Preguntar si eliminar reglas udev
read -p "¿Eliminar reglas UDEV del adaptador K+DCAN? (s/N): " delete_udev
if [[ "$delete_udev" =~ ^[sS]$ ]]; then
    log "Eliminando reglas UDEV..."
    rm -f /etc/udev/rules.d/99-kdcan.rules
    udevadm control --reload-rules
fi

echo ""
log "Desinstalación completada"
echo ""
