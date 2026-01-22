#!/bin/bash
#
# BMW Diagnostic Tool - Instalador para Debian/Ubuntu
# Este script instala las dependencias y configura el sistema
#

set -e

echo "================================================"
echo "  BMW Diagnostic Tool - Instalador Debian"
echo "================================================"
echo ""

# Verificar que se ejecuta como root
if [ "$EUID" -ne 0 ]; then
    echo "Error: Ejecuta este script como root (sudo)"
    echo "Uso: sudo ./install-debian.sh"
    exit 1
fi

# Obtener el usuario real (no root)
REAL_USER=${SUDO_USER:-$USER}

echo "[1/5] Actualizando repositorios..."
apt-get update -qq

echo "[2/5] Instalando dependencias..."
apt-get install -y -qq \
    libwebkit2gtk-4.1-0 \
    libgtk-3-0 \
    libayatana-appindicator3-1 \
    librsvg2-2 \
    libudev1 \
    > /dev/null

echo "[3/5] Configurando reglas udev para cable K+DCAN FTDI..."

# Crear regla udev para cable FTDI
cat > /etc/udev/rules.d/99-ftdi-kdcan.rules << 'EOF'
# Reglas udev para cable K+DCAN con chip FTDI
# Permite acceso al puerto serial sin necesidad de root

# FTDI FT232R (comun en cables K+DCAN)
SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6001", MODE="0666", GROUP="dialout", SYMLINK+="kdcan"

# FTDI FT232H
SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6014", MODE="0666", GROUP="dialout", SYMLINK+="kdcan"

# FTDI FT232BM
SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6010", MODE="0666", GROUP="dialout", SYMLINK+="kdcan"

# Generico FTDI
SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", MODE="0666", GROUP="dialout"
EOF

echo "[4/5] Recargando reglas udev..."
udevadm control --reload-rules
udevadm trigger

echo "[5/5] Agregando usuario '$REAL_USER' al grupo dialout..."
usermod -aG dialout "$REAL_USER"

echo ""
echo "================================================"
echo "  Instalacion completada!"
echo "================================================"
echo ""
echo "IMPORTANTE:"
echo "  1. Cierra sesion y vuelve a entrar para aplicar"
echo "     los permisos del grupo dialout"
echo ""
echo "  2. Conecta el cable K+DCAN y verifica con:"
echo "     ls -la /dev/ttyUSB*"
echo ""
echo "  3. El cable aparecera tambien como /dev/kdcan"
echo ""
echo "  4. Para ejecutar la aplicacion:"
echo "     - AppImage: ./BMW_Diag_0.1.0_amd64.AppImage"
echo "     - O instala el .deb: sudo dpkg -i BMW_Diag_0.1.0_amd64.deb"
echo ""
