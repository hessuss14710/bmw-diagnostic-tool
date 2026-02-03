# BMW Diagnostic Tool

Herramienta de diagnóstico para BMW E60/E90 con cable K+DCAN. Permite leer/borrar códigos de error, ver datos en vivo, gestionar el DPF y más.

![Tauri](https://img.shields.io/badge/Tauri-2.0-blue)
![React](https://img.shields.io/badge/React-19-61DAFB)
![Rust](https://img.shields.io/badge/Rust-1.77-orange)
![License](https://img.shields.io/badge/License-MIT-green)
![Tests](https://img.shields.io/badge/Tests-125%20passed-brightgreen)

---

## Características

- **Diagnóstico Multi-ECU**: DDE/DME, DSC, KOMBI, FRM, EGS
- **Lectura/Borrado DTCs**: Códigos de error con descripciones detalladas
- **Datos en vivo**: RPM, temperaturas, presiones, turbo, etc.
- **Gestión DPF**:
  - Estado del filtro (hollín, cenizas, presión diferencial)
  - Reset de contadores
  - Regeneración forzada
- **Base de datos**: Historial de vehículos y diagnósticos (SQLite)
- **Interfaz moderna**: Diseño BMW con componentes premium
- **Protocolos**: K-Line (KWP2000) y D-CAN (ISO 15765)

---

## Instalación Rápida (Debian/Ubuntu)

### Opción 1: Script automático (Recomendado)

```bash
wget https://github.com/hessuss14710/bmw-diagnostic-tool/releases/download/v0.1.0/install-bmw-diag.sh
chmod +x install-bmw-diag.sh
sudo ./install-bmw-diag.sh
```

El script instala todas las dependencias, configura el adaptador K+DCAN y la aplicación.

### Opción 2: Instalación manual del .deb

```bash
# Descargar
wget https://github.com/hessuss14710/bmw-diagnostic-tool/releases/download/v0.1.0/BMW.Diag_0.1.0_amd64.deb

# Instalar
sudo dpkg -i BMW.Diag_0.1.0_amd64.deb
sudo apt install -f -y

# Configurar permisos para el adaptador
sudo usermod -aG dialout $USER

# Reiniciar sesión
```

### Opción 3: AppImage (Portable)

```bash
wget https://github.com/hessuss14710/bmw-diagnostic-tool/releases/download/v0.1.0/BMW.Diag_0.1.0_amd64.AppImage
chmod +x BMW.Diag_0.1.0_amd64.AppImage
./BMW.Diag_0.1.0_amd64.AppImage
```

---

## Requisitos

### Hardware
| Componente | Especificación |
|------------|----------------|
| Adaptador | K+DCAN USB (FTDI FT232RL recomendado) |
| Vehículo | BMW E60/E90 series (2003-2010) |
| PC | x86_64, 2GB RAM mínimo |

### Software
| Sistema | Versión |
|---------|---------|
| Debian | 12 (Bookworm) |
| Ubuntu | 22.04+ |
| Kernel | 5.x+ |

### Adaptadores USB soportados
| Chip | Vendor ID | Product ID |
|------|-----------|------------|
| FTDI FT232RL | 0403 | 6001 |
| FTDI FT232R | 0403 | 6015 |
| CH340/CH341 | 1a86 | 7523 |
| CP2102 | 10c4 | ea60 |

---

## Instalación de Debian desde cero

Si tu PC tiene Windows y quieres instalar Debian:

### 1. Descargar Debian
```
https://cdimage.debian.org/debian-cd/current/amd64/iso-cd/debian-12.9.0-amd64-netinst.iso
```

### 2. Crear USB booteable
- Windows: Usar [Rufus](https://rufus.ie/)
- Linux: `sudo dd if=debian.iso of=/dev/sdX bs=4M`

### 3. Instalar Debian
1. Arrancar desde USB
2. Seleccionar "Graphical Install"
3. Idioma: Español
4. Particionado: "Guiado - usar todo el disco"
5. Software: GNOME + Utilidades estándar
6. Instalar GRUB

### 4. Después de instalar Debian
```bash
# Descargar e instalar BMW Diagnostic Tool
wget https://github.com/hessuss14710/bmw-diagnostic-tool/releases/download/v0.1.0/install-bmw-diag.sh
chmod +x install-bmw-diag.sh
sudo ./install-bmw-diag.sh
```

---

## Configuración del Adaptador K+DCAN

El script de instalación configura esto automáticamente. Si necesitas hacerlo manual:

```bash
# Crear reglas udev
sudo tee /etc/udev/rules.d/99-kdcan.rules << 'EOF'
SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6001", MODE="0666", GROUP="dialout", SYMLINK+="kdcan"
SUBSYSTEM=="tty", ATTRS{idVendor}=="1a86", ATTRS{idProduct}=="7523", MODE="0666", GROUP="dialout"
EOF

# Recargar reglas
sudo udevadm control --reload-rules
sudo udevadm trigger

# Añadir usuario al grupo dialout
sudo usermod -aG dialout $USER

# Reiniciar sesión para aplicar
```

### Verificar adaptador
```bash
# Si instalaste con el script:
kdcan-test

# Manual:
lsusb | grep -i "ft232\|ch340"
ls -la /dev/ttyUSB*
```

---

## Uso de la Aplicación

### Conexión básica
1. Conectar adaptador K+DCAN al USB del PC
2. Conectar cable OBD-II al vehículo
3. Poner contacto (posición II, sin arrancar)
4. Abrir BMW Diagnostic Tool
5. Seleccionar puerto (`/dev/ttyUSB0`)
6. Clic en "Connect"

### Leer códigos de error
1. Seleccionar ECU (ej: DDE para diésel)
2. Clic en "Read DTCs"
3. Los códigos aparecen con descripción

### Funciones DPF
1. Conectar a ECU **DDE**
2. Ir a pestaña **DPF**
3. Iniciar sesión extendida
4. Funciones disponibles:
   - Ver estado (hollín, cenizas, temperaturas)
   - Reset contador de cenizas
   - Reset modelo/adaptación
   - Regeneración forzada

### Datos en vivo
1. Conectar a ECU deseada
2. Ir a pestaña **Live Data**
3. Seleccionar parámetros a monitorear
4. Los datos se actualizan en tiempo real

---

## ECUs Soportadas

| ECU | Nombre | Dirección | Protocolo |
|-----|--------|-----------|-----------|
| DDE | Digital Diesel Electronics | 0x12 | K-Line/D-CAN |
| DME | Digital Motor Electronics | 0x12 | K-Line/D-CAN |
| DSC | Dynamic Stability Control | 0x00 | D-CAN |
| KOMBI | Instrument Cluster | 0x40 | K-Line |
| FRM | Footwell Module | 0x00 | D-CAN |
| EGS | Electronic Gearbox | 0x32 | K-Line |

---

## Estructura del Proyecto

```
/srv/taller/
├── app/                          # Aplicación Tauri
│   ├── src/                      # Frontend React
│   │   ├── components/
│   │   │   ├── ui/              # Componentes (Button, Card, Modal...)
│   │   │   ├── dpf/             # Panel DPF
│   │   │   ├── dtc/             # Panel diagnóstico
│   │   │   ├── ecu/             # Paneles ECU
│   │   │   ├── vehicles/        # Gestión vehículos
│   │   │   └── settings/        # Backup/Restore
│   │   └── hooks/               # useDatabase, useBMW, useDPF...
│   │
│   └── src-tauri/               # Backend Rust
│       └── src/
│           ├── database.rs      # SQLite persistence
│           ├── bmw_commands.rs  # Comandos diagnóstico
│           ├── kline.rs         # Protocolo K-Line
│           ├── dcan.rs          # Protocolo D-CAN
│           └── validators.rs    # Validación input
│
├── scripts/                      # Scripts instalación
│   ├── install-bmw-diag.sh
│   └── uninstall-bmw-diag.sh
│
├── server/                       # API Backend (opcional)
│   └── src/data/                # DTCs database JSON
│
└── usb-files/                   # Archivos para USB booteable
```

---

## Compilar desde Código Fuente

```bash
# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Instalar Node.js 20
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Instalar dependencias del sistema
sudo apt install -y \
  build-essential \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libusb-1.0-0-dev

# Clonar y compilar
git clone https://github.com/hessuss14710/bmw-diagnostic-tool.git
cd bmw-diagnostic-tool/app
npm install
npm run tauri build

# El .deb estará en:
# src-tauri/target/release/bundle/deb/
```

---

## Tests

```bash
# Frontend (73 tests)
cd app
npm run test:run

# Backend Rust (52 tests)
cd app/src-tauri
cargo test

# Total: 125 tests
```

---

## Solución de Problemas

### El adaptador no se detecta
```bash
# Verificar USB
lsusb | grep -i "ft232\|ch340"

# Cargar módulo
sudo modprobe ftdi_sio

# Ver errores
dmesg | tail -20
```

### Permiso denegado al puerto
```bash
# Verificar grupos
groups

# Si no está en dialout:
sudo usermod -aG dialout $USER
# Cerrar sesión y volver a entrar
```

### La aplicación no conecta con el vehículo
1. Verificar contacto puesto (posición II)
2. Verificar conexión OBD-II física
3. Probar otro puerto USB
4. Verificar que el adaptador está en modo BMW (no OBD genérico)

### Ver logs de la aplicación
```bash
bmw-diag 2>&1 | tee debug.log
```

---

## Advertencias de Seguridad

⚠️ **Regeneración DPF**:
- Eleva temperaturas del escape a >600°C
- Mantener vehículo en zona ventilada
- No tocar el tubo de escape
- Motor debe estar en marcha, vehículo parado

⚠️ **Borrado de DTCs**:
- No borra la causa del problema
- Algunos códigos vuelven si el problema persiste

⚠️ **Uso general**:
- No usar mientras se conduce
- Verificar conexiones antes de operar
- Hacer backup de datos antes de cambios

---

## Descargas

| Archivo | Descripción | Enlace |
|---------|-------------|--------|
| `install-bmw-diag.sh` | Script instalación | [Descargar](https://github.com/hessuss14710/bmw-diagnostic-tool/releases/download/v0.1.0/install-bmw-diag.sh) |
| `BMW.Diag_0.1.0_amd64.deb` | Paquete Debian | [Descargar](https://github.com/hessuss14710/bmw-diagnostic-tool/releases/download/v0.1.0/BMW.Diag_0.1.0_amd64.deb) |
| `BMW.Diag_0.1.0_amd64.AppImage` | Portable | [Descargar](https://github.com/hessuss14710/bmw-diagnostic-tool/releases/download/v0.1.0/BMW.Diag_0.1.0_amd64.AppImage) |

**Todas las releases**: https://github.com/hessuss14710/bmw-diagnostic-tool/releases

---

## Licencia

MIT License - Ver [LICENSE](LICENSE)

---

## Créditos

Desarrollado con:
- [Tauri 2.0](https://tauri.app/) - Framework desktop
- [React 19](https://react.dev/) - UI Framework
- [Rust](https://www.rust-lang.org/) - Backend
- [SQLite](https://sqlite.org/) - Base de datos
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite bindings

---

**GitHub**: https://github.com/hessuss14710/bmw-diagnostic-tool
