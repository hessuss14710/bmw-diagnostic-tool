#!/bin/bash
#
# ╔═══════════════════════════════════════════════════════════════════════════╗
# ║           BMW Diagnostic Tool - Script de Instalación Automática          ║
# ╠═══════════════════════════════════════════════════════════════════════════╣
# ║  Este script instala y configura todo lo necesario para usar la           ║
# ║  herramienta de diagnóstico BMW en Debian 12 (Bookworm)                   ║
# ║                                                                           ║
# ║  Uso: sudo ./install-bmw-diag.sh                                          ║
# ║                                                                           ║
# ║  Opciones:                                                                ║
# ║    --no-reboot    No reiniciar automáticamente al finalizar               ║
# ║    --dev          Instalar herramientas de desarrollo adicionales         ║
# ║    --help         Mostrar esta ayuda                                      ║
# ╚═══════════════════════════════════════════════════════════════════════════╝
#

set -e

# ============================================================================
# CONFIGURACIÓN
# ============================================================================

VERSION="1.0.0"
GITHUB_REPO="hessuss14710/bmw-diagnostic-tool"
APP_NAME="BMW Diag"
LOG_FILE="/var/log/bmw-diag-install.log"

# Colores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Opciones
AUTO_REBOOT=true
INSTALL_DEV_TOOLS=false

# ============================================================================
# FUNCIONES AUXILIARES
# ============================================================================

log() {
    echo -e "${GREEN}[✓]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
}

warn() {
    echo -e "${YELLOW}[!]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $1" >> "$LOG_FILE"
}

error() {
    echo -e "${RED}[✗]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $1" >> "$LOG_FILE"
}

info() {
    echo -e "${BLUE}[i]${NC} $1"
}

header() {
    echo ""
    echo -e "${CYAN}${BOLD}══════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}${BOLD}  $1${NC}"
    echo -e "${CYAN}${BOLD}══════════════════════════════════════════════════════════════${NC}"
    echo ""
}

spinner() {
    local pid=$1
    local delay=0.1
    local spinstr='|/-\'
    while [ "$(ps a | awk '{print $1}' | grep $pid)" ]; do
        local temp=${spinstr#?}
        printf " [%c]  " "$spinstr"
        local spinstr=$temp${spinstr%"$temp"}
        sleep $delay
        printf "\b\b\b\b\b\b"
    done
    printf "    \b\b\b\b"
}

check_root() {
    if [ "$EUID" -ne 0 ]; then
        error "Este script debe ejecutarse como root"
        echo "Uso: sudo $0"
        exit 1
    fi
}

get_real_user() {
    if [ -n "$SUDO_USER" ]; then
        echo "$SUDO_USER"
    else
        echo "$USER"
    fi
}

get_real_home() {
    local user=$(get_real_user)
    eval echo ~$user
}

show_help() {
    echo "BMW Diagnostic Tool - Instalador v${VERSION}"
    echo ""
    echo "Uso: sudo $0 [opciones]"
    echo ""
    echo "Opciones:"
    echo "  --no-reboot    No reiniciar automáticamente al finalizar"
    echo "  --dev          Instalar herramientas de desarrollo (Rust, Node.js)"
    echo "  --help         Mostrar esta ayuda"
    echo ""
    echo "Ejemplo:"
    echo "  sudo $0 --no-reboot"
    echo ""
}

show_banner() {
    clear
    echo -e "${BLUE}"
    cat << 'EOF'
    ____  __  ____          ____  _                             __  _
   / __ )/  |/  / |        / / / | |                           / /_(_)
  / __  / /|_/ /| |  /|   / / /  | |     ___   __ _ _ __   ___| __| | ___
 / /_/ / /  / / | | / |  / / /   | |    / _ \ / _` | '_ \ / _ \ |_| |/ __|
/_____/_/  /_/  |___/|__/_/_/    |_|   |  __/| (_| | | | |  __/\__|_|\__ \
                                       |___|  \__, |_| |_|\___|\__|_|___/
                                               __/ |
     D I A G N O S T I C   T O O L            |___/           v0.1.0
EOF
    echo -e "${NC}"
    echo ""
    echo -e "${BOLD}Instalador Automático v${VERSION}${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
}

# ============================================================================
# PARSEAR ARGUMENTOS
# ============================================================================

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --no-reboot)
                AUTO_REBOOT=false
                shift
                ;;
            --dev)
                INSTALL_DEV_TOOLS=true
                shift
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                error "Opción desconocida: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

# ============================================================================
# VERIFICACIONES PREVIAS
# ============================================================================

check_system() {
    header "Verificando Sistema"

    # Verificar Debian
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        if [[ "$ID" != "debian" && "$ID_LIKE" != *"debian"* ]]; then
            error "Este script está diseñado para Debian/Ubuntu"
            exit 1
        fi
        log "Sistema detectado: $PRETTY_NAME"
    else
        warn "No se pudo detectar el sistema operativo"
    fi

    # Verificar arquitectura
    ARCH=$(uname -m)
    if [ "$ARCH" != "x86_64" ]; then
        error "Arquitectura no soportada: $ARCH (se requiere x86_64)"
        exit 1
    fi
    log "Arquitectura: $ARCH"

    # Verificar conexión a internet
    if ping -c 1 google.com &> /dev/null; then
        log "Conexión a internet: OK"
    else
        error "No hay conexión a internet"
        exit 1
    fi

    # Verificar espacio en disco
    FREE_SPACE=$(df -BG / | awk 'NR==2 {print $4}' | tr -d 'G')
    if [ "$FREE_SPACE" -lt 5 ]; then
        error "Espacio insuficiente en disco (mínimo 5GB, disponible: ${FREE_SPACE}GB)"
        exit 1
    fi
    log "Espacio disponible: ${FREE_SPACE}GB"

    # Obtener información del usuario
    REAL_USER=$(get_real_user)
    REAL_HOME=$(get_real_home)
    log "Usuario: $REAL_USER"
    log "Home: $REAL_HOME"
}

# ============================================================================
# ACTUALIZACIÓN DEL SISTEMA
# ============================================================================

update_system() {
    header "Actualizando Sistema"

    log "Actualizando lista de paquetes..."
    apt-get update -qq

    log "Actualizando paquetes instalados..."
    DEBIAN_FRONTEND=noninteractive apt-get upgrade -y -qq

    log "Sistema actualizado"
}

# ============================================================================
# INSTALACIÓN DE DEPENDENCIAS
# ============================================================================

install_dependencies() {
    header "Instalando Dependencias"

    log "Instalando paquetes esenciales..."

    PACKAGES=(
        # Utilidades básicas
        curl
        wget
        git
        unzip

        # Bibliotecas para Tauri/GTK
        libgtk-3-0
        libwebkit2gtk-4.1-0
        libayatana-appindicator3-1
        librsvg2-2

        # USB y serie
        usbutils
        libusb-1.0-0
        libudev1

        # Herramientas de diagnóstico
        minicom
        screen
        picocom

        # Extras útiles
        policykit-1
        xdg-utils
    )

    for pkg in "${PACKAGES[@]}"; do
        if dpkg -l | grep -q "^ii  $pkg "; then
            info "  $pkg ya instalado"
        else
            log "  Instalando $pkg..."
            apt-get install -y -qq "$pkg" 2>/dev/null || warn "No se pudo instalar $pkg"
        fi
    done

    log "Dependencias instaladas"
}

# ============================================================================
# HERRAMIENTAS DE DESARROLLO (OPCIONAL)
# ============================================================================

install_dev_tools() {
    if [ "$INSTALL_DEV_TOOLS" = false ]; then
        return
    fi

    header "Instalando Herramientas de Desarrollo"

    # Rust
    if ! command -v rustc &> /dev/null; then
        log "Instalando Rust..."
        su - "$REAL_USER" -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
        log "Rust instalado"
    else
        info "Rust ya está instalado"
    fi

    # Node.js
    if ! command -v node &> /dev/null; then
        log "Instalando Node.js 20..."
        curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
        apt-get install -y -qq nodejs
        log "Node.js instalado: $(node --version)"
    else
        info "Node.js ya está instalado: $(node --version)"
    fi

    # Dependencias de compilación
    log "Instalando dependencias de compilación..."
    apt-get install -y -qq \
        build-essential \
        pkg-config \
        libssl-dev \
        libgtk-3-dev \
        libwebkit2gtk-4.1-dev \
        libayatana-appindicator3-dev \
        librsvg2-dev \
        libusb-1.0-0-dev \
        libudev-dev

    log "Herramientas de desarrollo instaladas"
}

# ============================================================================
# CONFIGURACIÓN UDEV PARA K+DCAN
# ============================================================================

configure_udev() {
    header "Configurando Reglas UDEV"

    log "Creando reglas para adaptador K+DCAN..."

    cat > /etc/udev/rules.d/99-kdcan.rules << 'EOF'
# ============================================================================
# BMW K+DCAN USB Adapter Rules
# ============================================================================

# FTDI FT232RL (Adaptador K+DCAN estándar)
SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6001", \
    MODE="0666", GROUP="dialout", SYMLINK+="kdcan", \
    ENV{ID_MM_DEVICE_IGNORE}="1"

# FTDI FT232R (Variante)
SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6015", \
    MODE="0666", GROUP="dialout", SYMLINK+="kdcan", \
    ENV{ID_MM_DEVICE_IGNORE}="1"

# FTDI FT232H
SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6014", \
    MODE="0666", GROUP="dialout", SYMLINK+="kdcan", \
    ENV{ID_MM_DEVICE_IGNORE}="1"

# CH340/CH341 (Adaptadores genéricos chinos)
SUBSYSTEM=="tty", ATTRS{idVendor}=="1a86", ATTRS{idProduct}=="7523", \
    MODE="0666", GROUP="dialout", \
    ENV{ID_MM_DEVICE_IGNORE}="1"

# CP2102 (Silicon Labs)
SUBSYSTEM=="tty", ATTRS{idVendor}=="10c4", ATTRS{idProduct}=="ea60", \
    MODE="0666", GROUP="dialout", \
    ENV{ID_MM_DEVICE_IGNORE}="1"

# Permitir acceso USB directo
SUBSYSTEM=="usb", ATTRS{idVendor}=="0403", MODE="0666"
SUBSYSTEM=="usb", ATTRS{idVendor}=="1a86", MODE="0666"
SUBSYSTEM=="usb", ATTRS{idVendor}=="10c4", MODE="0666"

# ============================================================================
# Deshabilitar ModemManager para estos dispositivos
# (evita que interfiera con la comunicación serie)
# ============================================================================
ATTRS{idVendor}=="0403", ENV{ID_MM_DEVICE_IGNORE}="1"
ATTRS{idVendor}=="1a86", ENV{ID_MM_DEVICE_IGNORE}="1"
ATTRS{idVendor}=="10c4", ENV{ID_MM_DEVICE_IGNORE}="1"
EOF

    log "Recargando reglas udev..."
    udevadm control --reload-rules
    udevadm trigger

    log "Reglas UDEV configuradas"
}

# ============================================================================
# CONFIGURACIÓN DE USUARIO
# ============================================================================

configure_user() {
    header "Configurando Usuario"

    REAL_USER=$(get_real_user)

    # Añadir a grupo dialout
    if groups "$REAL_USER" | grep -q dialout; then
        info "Usuario $REAL_USER ya está en grupo dialout"
    else
        log "Añadiendo $REAL_USER al grupo dialout..."
        usermod -aG dialout "$REAL_USER"
        log "Usuario añadido al grupo dialout"
    fi

    # Añadir a grupo plugdev (para acceso USB)
    if groups "$REAL_USER" | grep -q plugdev; then
        info "Usuario $REAL_USER ya está en grupo plugdev"
    else
        log "Añadiendo $REAL_USER al grupo plugdev..."
        usermod -aG plugdev "$REAL_USER"
    fi

    log "Usuario configurado"
}

# ============================================================================
# DESCARGAR E INSTALAR BMW DIAGNOSTIC TOOL
# ============================================================================

install_bmw_diag() {
    header "Instalando BMW Diagnostic Tool"

    REAL_HOME=$(get_real_home)
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    # Intentar descargar desde GitHub releases
    log "Buscando última versión..."

    # Obtener URL de la última release
    RELEASE_URL="https://api.github.com/repos/${GITHUB_REPO}/releases/latest"

    if RELEASE_INFO=$(curl -s "$RELEASE_URL" 2>/dev/null); then
        DEB_URL=$(echo "$RELEASE_INFO" | grep -o '"browser_download_url": *"[^"]*\.deb"' | head -1 | cut -d'"' -f4)

        if [ -n "$DEB_URL" ]; then
            log "Descargando desde GitHub..."
            wget -q --show-progress -O bmw-diag.deb "$DEB_URL"
        fi
    fi

    # Si no se pudo descargar, buscar en el servidor local
    if [ ! -f bmw-diag.deb ]; then
        warn "No se pudo descargar desde GitHub"

        # Buscar .deb local
        LOCAL_DEB=$(find /srv/taller -name "*.deb" -path "*/bundle/deb/*" 2>/dev/null | head -1)

        if [ -n "$LOCAL_DEB" ]; then
            log "Usando .deb local: $LOCAL_DEB"
            cp "$LOCAL_DEB" bmw-diag.deb
        else
            error "No se encontró el paquete .deb"
            error "Descarga manualmente desde: https://github.com/${GITHUB_REPO}/releases"
            cd /
            rm -rf "$TEMP_DIR"
            return 1
        fi
    fi

    # Instalar el .deb
    log "Instalando paquete..."
    dpkg -i bmw-diag.deb 2>/dev/null || true
    apt-get install -f -y -qq

    # Verificar instalación
    if command -v bmw-diag &> /dev/null || [ -f /usr/bin/bmw-diag ]; then
        log "BMW Diagnostic Tool instalado correctamente"
    else
        warn "La instalación puede requerir revisión manual"
    fi

    # Limpiar
    cd /
    rm -rf "$TEMP_DIR"

    log "Instalación completada"
}

# ============================================================================
# CREAR SCRIPTS DE UTILIDAD
# ============================================================================

create_utilities() {
    header "Creando Utilidades"

    REAL_HOME=$(get_real_home)
    REAL_USER=$(get_real_user)

    # Script de diagnóstico del adaptador
    log "Creando script de diagnóstico..."

    cat > /usr/local/bin/kdcan-test << 'EOF'
#!/bin/bash
#
# K+DCAN Adapter Diagnostic Tool
#

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║          K+DCAN Adapter - Diagnóstico                     ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Colores
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass() { echo -e "${GREEN}[✓]${NC} $1"; }
fail() { echo -e "${RED}[✗]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }

echo "1. Dispositivos USB detectados:"
echo "   ─────────────────────────────"
USB_FOUND=false
while IFS= read -r line; do
    if echo "$line" | grep -qi "ft232\|ch340\|ch341\|cp210\|serial\|0403\|1a86\|10c4"; then
        echo "   $line"
        USB_FOUND=true
    fi
done < <(lsusb 2>/dev/null)

if [ "$USB_FOUND" = false ]; then
    fail "No se detectó ningún adaptador USB-Serie"
else
    pass "Adaptador USB detectado"
fi
echo ""

echo "2. Puertos serie disponibles:"
echo "   ─────────────────────────────"
if ls /dev/ttyUSB* 2>/dev/null; then
    pass "Puertos ttyUSB encontrados"
else
    fail "No hay puertos /dev/ttyUSB*"
fi
echo ""

echo "3. Symlink /dev/kdcan:"
echo "   ─────────────────────────────"
if [ -L /dev/kdcan ]; then
    TARGET=$(readlink -f /dev/kdcan)
    pass "/dev/kdcan -> $TARGET"
else
    warn "/dev/kdcan no existe (se creará al conectar el adaptador)"
fi
echo ""

echo "4. Permisos de usuario:"
echo "   ─────────────────────────────"
if groups | grep -q dialout; then
    pass "Usuario en grupo 'dialout'"
else
    fail "Usuario NO está en grupo 'dialout'"
    echo "   Ejecuta: sudo usermod -aG dialout \$USER"
fi
echo ""

echo "5. Test de acceso al puerto:"
echo "   ─────────────────────────────"
PORT=$(ls /dev/ttyUSB* 2>/dev/null | head -1)
if [ -n "$PORT" ]; then
    if [ -r "$PORT" ] && [ -w "$PORT" ]; then
        pass "$PORT es accesible (lectura/escritura)"
    else
        fail "$PORT no es accesible"
        echo "   Permisos actuales: $(ls -la $PORT)"
    fi
else
    warn "No hay puerto para probar"
fi
echo ""

echo "6. Módulos del kernel:"
echo "   ─────────────────────────────"
for mod in ftdi_sio ch341 cp210x usbserial; do
    if lsmod | grep -q "^$mod"; then
        pass "Módulo $mod cargado"
    fi
done
echo ""

echo "═══════════════════════════════════════════════════════════"
echo "Diagnóstico completado"
echo ""
EOF
    chmod +x /usr/local/bin/kdcan-test

    # Script para monitor serie rápido
    log "Creando script de monitor serie..."

    cat > /usr/local/bin/kdcan-monitor << 'EOF'
#!/bin/bash
#
# Quick serial monitor for K+DCAN
#

PORT=${1:-/dev/ttyUSB0}
BAUD=${2:-10400}

if [ ! -e "$PORT" ]; then
    echo "Error: Puerto $PORT no encontrado"
    echo "Puertos disponibles:"
    ls /dev/ttyUSB* 2>/dev/null || echo "  Ninguno"
    exit 1
fi

echo "Conectando a $PORT @ ${BAUD} baud..."
echo "Presiona Ctrl+A, K para salir"
echo ""

screen "$PORT" "$BAUD"
EOF
    chmod +x /usr/local/bin/kdcan-monitor

    # Crear acceso directo en escritorio
    log "Creando acceso directo en escritorio..."

    DESKTOP_DIR="$REAL_HOME/Escritorio"
    [ ! -d "$DESKTOP_DIR" ] && DESKTOP_DIR="$REAL_HOME/Desktop"
    [ ! -d "$DESKTOP_DIR" ] && mkdir -p "$DESKTOP_DIR"

    cat > "$DESKTOP_DIR/BMW Diagnostic Tool.desktop" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=BMW Diagnostic Tool
Comment=Herramienta de diagnóstico para BMW
Exec=bmw-diag
Icon=applications-system
Terminal=false
Categories=Utility;Development;
StartupNotify=true
EOF
    chmod +x "$DESKTOP_DIR/BMW Diagnostic Tool.desktop"
    chown "$REAL_USER:$REAL_USER" "$DESKTOP_DIR/BMW Diagnostic Tool.desktop"

    # Permitir ejecución del .desktop en GNOME
    if command -v gio &> /dev/null; then
        su - "$REAL_USER" -c "gio set '$DESKTOP_DIR/BMW Diagnostic Tool.desktop' metadata::trusted true" 2>/dev/null || true
    fi

    log "Utilidades creadas"
    info "  - kdcan-test    : Diagnóstico del adaptador"
    info "  - kdcan-monitor : Monitor serie rápido"
}

# ============================================================================
# CONFIGURACIÓN FINAL
# ============================================================================

final_configuration() {
    header "Configuración Final"

    # Deshabilitar ModemManager si está instalado (interfiere con puertos serie)
    if systemctl is-active --quiet ModemManager; then
        log "Deshabilitando ModemManager (interfiere con puertos serie)..."
        systemctl stop ModemManager
        systemctl disable ModemManager
    fi

    # Limpiar
    log "Limpiando paquetes innecesarios..."
    apt-get autoremove -y -qq
    apt-get clean

    log "Configuración final completada"
}

# ============================================================================
# RESUMEN FINAL
# ============================================================================

show_summary() {
    REAL_USER=$(get_real_user)

    echo ""
    echo -e "${GREEN}${BOLD}"
    echo "╔═══════════════════════════════════════════════════════════════════════════╗"
    echo "║                     ¡INSTALACIÓN COMPLETADA!                              ║"
    echo "╚═══════════════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    echo ""
    echo -e "${BOLD}Resumen:${NC}"
    echo "  ✓ Sistema actualizado"
    echo "  ✓ Dependencias instaladas"
    echo "  ✓ Reglas UDEV configuradas"
    echo "  ✓ Usuario $REAL_USER configurado"
    echo "  ✓ BMW Diagnostic Tool instalado"
    echo "  ✓ Utilidades creadas"
    echo ""
    echo -e "${BOLD}Comandos útiles:${NC}"
    echo "  kdcan-test        Diagnosticar adaptador K+DCAN"
    echo "  kdcan-monitor     Monitor serie rápido"
    echo "  bmw-diag          Iniciar aplicación"
    echo ""
    echo -e "${BOLD}Próximos pasos:${NC}"
    echo "  1. Reiniciar el sistema (necesario para aplicar grupos)"
    echo "  2. Conectar el adaptador K+DCAN"
    echo "  3. Ejecutar 'kdcan-test' para verificar"
    echo "  4. Iniciar 'BMW Diagnostic Tool'"
    echo ""
    echo -e "${YELLOW}IMPORTANTE: Debes reiniciar para que los cambios de grupo tengan efecto${NC}"
    echo ""

    if [ "$AUTO_REBOOT" = true ]; then
        echo -e "${BOLD}El sistema se reiniciará en 10 segundos...${NC}"
        echo "Presiona Ctrl+C para cancelar"
        sleep 10
        reboot
    else
        echo "Ejecuta 'sudo reboot' cuando estés listo para reiniciar"
    fi
}

# ============================================================================
# MAIN
# ============================================================================

main() {
    # Parsear argumentos
    parse_args "$@"

    # Mostrar banner
    show_banner

    # Iniciar log
    echo "=== BMW Diagnostic Tool Installation ===" > "$LOG_FILE"
    echo "Started: $(date)" >> "$LOG_FILE"
    echo "User: $(get_real_user)" >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"

    # Verificar root
    check_root

    # Ejecutar pasos de instalación
    check_system
    update_system
    install_dependencies
    install_dev_tools
    configure_udev
    configure_user
    install_bmw_diag
    create_utilities
    final_configuration

    # Log final
    echo "" >> "$LOG_FILE"
    echo "Completed: $(date)" >> "$LOG_FILE"

    # Mostrar resumen
    show_summary
}

# Ejecutar
main "$@"
