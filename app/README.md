# BMW Diagnostic Tool

Herramienta de diagnóstico para BMW E60 con cable K+DCAN FTDI. Permite leer/borrar códigos de error, ver datos en vivo y gestionar el DPF (filtro de partículas diésel).

![Tauri](https://img.shields.io/badge/Tauri-2.0-blue)
![React](https://img.shields.io/badge/React-19-61DAFB)
![Rust](https://img.shields.io/badge/Rust-1.77-orange)
![License](https://img.shields.io/badge/License-MIT-green)

## Características

- **Conexión K+DCAN**: Soporte para cables con chip FTDI FT232
- **Protocolos**: K-Line (ISO 14230 KWP2000) y D-CAN (ISO 15765)
- **Lectura de DTCs**: Códigos de error con descripción
- **Borrado de DTCs**: Limpieza de códigos almacenados
- **Datos en vivo**: RPM, temperatura, presiones, etc.
- **Funciones DPF**:
  - Leer estado del DPF (hollín, cenizas, temperaturas)
  - Reset contador de cenizas
  - Reset modelo/adaptación DPF
  - Registrar DPF nuevo instalado
  - Regeneración forzada

## Requisitos

### Hardware
- Cable K+DCAN con chip FTDI (FT232R o FT232H)
- BMW E60 (2003-2010) con motor diésel

### Software
- Linux (Debian/Ubuntu recomendado)
- Rust 1.77+
- Node.js 18+

## Instalación

### Desde AppImage (Recomendado)

```bash
# Descargar el AppImage desde releases
chmod +x BMW_Diag_*.AppImage
./BMW_Diag_*.AppImage
```

### Compilar desde fuente

```bash
# Clonar repositorio
git clone https://github.com/tu-usuario/bmw-diag.git
cd bmw-diag/app

# Instalar dependencias
npm install

# Compilar
npm run tauri build
```

## Configuración del cable K+DCAN

En Linux, es necesario configurar los permisos del puerto serial:

```bash
# Crear regla udev para FTDI
sudo tee /etc/udev/rules.d/99-ftdi-kdcan.rules << 'EOF'
SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", MODE="0666", GROUP="dialout"
EOF

# Recargar reglas
sudo udevadm control --reload-rules
sudo udevadm trigger

# Añadir usuario al grupo dialout
sudo usermod -aG dialout $USER
```

Reiniciar sesión para aplicar los cambios.

## Uso

1. Conectar el cable K+DCAN al puerto USB
2. Conectar el cable OBD al coche
3. Poner el contacto (no arrancar motor)
4. Abrir la aplicación
5. Seleccionar puerto `/dev/ttyUSB0` y conectar
6. Seleccionar ECU (DDE para diésel)
7. Inicializar comunicación

### Funciones DPF

Para usar las funciones de DPF:

1. Seleccionar ECU: **DDE** (Digital Diesel Electronics)
2. Inicializar comunicación
3. Ir a pestaña **DPF**
4. Iniciar sesión extendida
5. Usar las funciones de reset según necesidad

## Estructura del proyecto

```
/srv/taller/
├── app/                    # Aplicación Tauri
│   ├── src/               # Frontend React
│   │   ├── components/    # Componentes UI
│   │   │   ├── dpf/      # Panel DPF
│   │   │   ├── dtc/      # Panel diagnóstico
│   │   │   └── live-data/ # Datos en vivo
│   │   └── hooks/        # Hooks (useBMW, useDPF, etc.)
│   └── src-tauri/        # Backend Rust
│       └── src/
│           ├── bmw.rs           # Tipos y constantes BMW
│           ├── bmw_commands.rs  # Comandos Tauri
│           ├── kline.rs         # Protocolo K-Line
│           └── dcan.rs          # Protocolo D-CAN
├── daemon-ftdi/           # Daemon FTDI (opcional)
└── dist/                  # Archivos distribuibles
    ├── BMW Diag_*.AppImage
    ├── install-debian.sh
    └── INSTRUCCIONES.txt
```

## Protocolos soportados

| Protocolo | Velocidad | Uso |
|-----------|-----------|-----|
| K-Line ISO 14230 | 10400 baud | ECUs antiguas, body modules |
| D-CAN ISO 15765 | 500 kbaud | ECUs modernas (E60, E90, etc.) |

## ECUs soportadas

| ECU | Descripción | Dirección |
|-----|-------------|-----------|
| DDE | Digital Diesel Electronics | 0x12 |
| DME | Digital Motor Electronics | 0x12 |
| EGS | Transmisión | 0x32 |
| DSC | Control estabilidad | 0x44 |
| Airbag | MRS | 0x4A |

## Funciones DPF - IDs de rutina

| Función | ID Principal | ID Alternativo |
|---------|--------------|----------------|
| Reset cenizas | 0xA091 | 0x0061 |
| Reset modelo | 0xA092 | 0x0062 |
| DPF nuevo | 0xA093 | 0x0063 |
| Regeneración | 0xA094 | 0x0064 |

> Nota: Los IDs pueden variar según la versión del DDE

## Advertencias

- La regeneración forzada eleva temperaturas a >600°C
- No tocar el escape durante/después de regeneración
- Vehículo debe estar parado con motor en marcha
- Usar en zona bien ventilada

## Licencia

MIT License - Ver [LICENSE](LICENSE)

## Créditos

Desarrollado con:
- [Tauri](https://tauri.app/) - Framework desktop
- [React](https://react.dev/) - UI
- [Rust](https://www.rust-lang.org/) - Backend
- [serialport-rs](https://github.com/serialport/serialport-rs) - Comunicación serial
