#!/bin/bash
#
# BMW DIAGNOSTIC TOOL - INSTALADOR RÁPIDO
#
# Uso: sudo ./INSTALAR.sh
#

clear
echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║         BMW DIAGNOSTIC TOOL - INSTALACIÓN AUTOMÁTICA          ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# Verificar root
if [ "$EUID" -ne 0 ]; then
    echo "❌ Error: Ejecutar como root"
    echo ""
    echo "   Uso correcto:  sudo ./INSTALAR.sh"
    echo ""
    exit 1
fi

# Obtener directorio del script
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DEB_FILE="$SCRIPT_DIR/BMW Diag_0.1.0_amd64.deb"

# Verificar que existe el .deb
if [ ! -f "$DEB_FILE" ]; then
    echo "❌ Error: No se encuentra el archivo .deb"
    echo "   Buscando en: $DEB_FILE"
    echo ""
    echo "   Asegúrate de que INSTALAR.sh y BMW Diag_0.1.0_amd64.deb"
    echo "   están en la misma carpeta."
    exit 1
fi

# Usuario real (no root)
REAL_USER=${SUDO_USER:-$USER}
echo "👤 Usuario: $REAL_USER"
echo ""

# Paso 1: Actualizar sistema
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 [1/5] Actualizando sistema..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
apt-get update -qq
apt-get upgrade -y -qq
echo "✅ Sistema actualizado"
echo ""

# Paso 2: Instalar dependencias
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 [2/5] Instalando dependencias..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
apt-get install -y -qq \
    libgtk-3-0 \
    libwebkit2gtk-4.1-0 \
    libayatana-appindicator3-1 \
    usbutils \
    wget \
    curl
echo "✅ Dependencias instaladas"
echo ""

# Paso 3: Instalar BMW Diagnostic Tool
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 [3/5] Instalando BMW Diagnostic Tool..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
dpkg -i "$DEB_FILE" 2>/dev/null || apt-get install -f -y -qq
echo "✅ Aplicación instalada"
echo ""

# Paso 4: Configurar adaptador K+DCAN
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔧 [4/5] Configurando adaptador K+DCAN..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Crear reglas udev
cat > /etc/udev/rules.d/99-kdcan.rules << 'EOF'
# FTDI FT232RL (K+DCAN estándar)
SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6001", MODE="0666", GROUP="dialout", SYMLINK+="kdcan"
SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6015", MODE="0666", GROUP="dialout", SYMLINK+="kdcan"
# CH340 (adaptadores genéricos)
SUBSYSTEM=="tty", ATTRS{idVendor}=="1a86", ATTRS{idProduct}=="7523", MODE="0666", GROUP="dialout"
# Deshabilitar ModemManager
ATTRS{idVendor}=="0403", ENV{ID_MM_DEVICE_IGNORE}="1"
ATTRS{idVendor}=="1a86", ENV{ID_MM_DEVICE_IGNORE}="1"
EOF

udevadm control --reload-rules
udevadm trigger
echo "✅ Reglas UDEV configuradas"
echo ""

# Paso 5: Configurar usuario
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "👤 [5/5] Configurando permisos de usuario..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
usermod -aG dialout "$REAL_USER"
usermod -aG plugdev "$REAL_USER"
echo "✅ Usuario $REAL_USER añadido a grupos dialout y plugdev"
echo ""

# Crear script de diagnóstico rápido
cat > /usr/local/bin/kdcan-test << 'TESTSCRIPT'
#!/bin/bash
echo "=== K+DCAN Adapter Test ==="
echo ""
echo "USB Devices:"
lsusb | grep -i "ft232\|ch340\|0403\|1a86" || echo "  No adapter found"
echo ""
echo "Serial Ports:"
ls -la /dev/ttyUSB* 2>/dev/null || echo "  No /dev/ttyUSB* found"
echo ""
echo "User Groups:"
groups | grep -q dialout && echo "  ✓ dialout OK" || echo "  ✗ NOT in dialout (reboot needed)"
TESTSCRIPT
chmod +x /usr/local/bin/kdcan-test

# Limpiar
apt-get autoremove -y -qq
apt-get clean

# Finalizar
echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║              ✅ INSTALACIÓN COMPLETADA                        ║"
echo "╠═══════════════════════════════════════════════════════════════╣"
echo "║                                                               ║"
echo "║  Próximos pasos:                                              ║"
echo "║                                                               ║"
echo "║  1. REINICIAR el sistema (obligatorio)                        ║"
echo "║     $ sudo reboot                                             ║"
echo "║                                                               ║"
echo "║  2. Conectar adaptador K+DCAN                                 ║"
echo "║                                                               ║"
echo "║  3. Verificar adaptador:                                      ║"
echo "║     $ kdcan-test                                              ║"
echo "║                                                               ║"
echo "║  4. Iniciar aplicación:                                       ║"
echo "║     - Desde menú: 'BMW Diagnostic Tool'                       ║"
echo "║     - Desde terminal: bmw-diag                                ║"
echo "║                                                               ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

read -p "¿Reiniciar ahora? (s/N): " respuesta
if [[ "$respuesta" =~ ^[sS]$ ]]; then
    echo "Reiniciando en 3 segundos..."
    sleep 3
    reboot
fi
