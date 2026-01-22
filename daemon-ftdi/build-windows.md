# Compilar BMW Diagnostic Daemon para Windows

## Requisitos Previos

### 1. Instalar Rust
```powershell
# Descargar e instalar desde:
# https://rustup.rs/
# O ejecutar en PowerShell:
winget install Rustlang.Rust.GNU
```

### 2. Instalar FTDI D2XX Drivers
1. Descargar de: https://ftdichip.com/drivers/d2xx-drivers/
2. Descargar "CDM v2.12.36.4 WHQL Certified" (o versión más reciente)
3. Instalar el driver
4. Verificar que el cable K+DCAN aparece en Administrador de Dispositivos

### 3. Descargar librerías D2XX
1. Ir a: https://ftdichip.com/software-examples/d2xx-library/
2. Descargar "D2XX Library" para Windows
3. Extraer `ftd2xx.lib` y `ftd2xx64.lib` a `C:\ftdi\` (o cualquier carpeta)

## Compilar

```powershell
# Clonar o copiar el proyecto
cd C:\bmw-diag-daemon

# Configurar variable de entorno para FTDI
$env:FTD2XX_DIR = "C:\ftdi"

# Compilar en modo release
cargo build --release

# El ejecutable estará en:
# target\release\bmw-diag-daemon.exe
```

## Ejecutar

```powershell
# Ejecutar el daemon
.\target\release\bmw-diag-daemon.exe

# Debería mostrar:
# ╔═══════════════════════════════════════════════════════╗
# ║     BMW Diagnostic Daemon v1.0 - FTDI D2XX            ║
# ║     High-Precision K-Line Communication               ║
# ╚═══════════════════════════════════════════════════════╝
#
# Found 1 FTDI device(s):
#   [0] FT232R USB UART - A12345
#
# WebSocket server listening on ws://127.0.0.1:3003
```

## Uso desde Web

1. Abrir https://taller.giormo.com/diagnostico-daemon.html
2. El navegador se conecta a ws://localhost:3003
3. El daemon maneja toda la comunicación con el coche

## Comandos WebSocket

```json
// Listar dispositivos
{"cmd": "list_devices"}

// Conectar a dispositivo 0
{"cmd": "connect", "data": {"device_index": 0}}

// Inicializar ECU DME (0x12) con fast init
{"cmd": "init_ecu", "data": {"address": 18, "fast": true}}

// Leer códigos de error
{"cmd": "read_dtcs"}

// Borrar códigos
{"cmd": "clear_dtcs"}

// Leer PID (RPM = 0x0C)
{"cmd": "read_pid", "data": {"pid": 12}}

// Leer múltiples PIDs
{"cmd": "read_pids", "data": {"pids": [12, 5, 13, 17]}}
```

## Solución de Problemas

### "No FTDI devices found"
1. Verificar que el cable está conectado
2. Abrir Administrador de Dispositivos
3. Buscar "Puertos (COM y LPT)" → debe aparecer "USB Serial Port"
4. Si aparece con signo de exclamación, reinstalar drivers

### "Access Denied"
1. Cerrar cualquier programa que use el cable (INPA, etc.)
2. Ejecutar el daemon como Administrador

### Latencia alta
1. Verificar que no hay otros programas accediendo al USB
2. Usar puerto USB directo (no hub)
3. Ejecutar con prioridad alta (ver abajo)

## Notas sobre Precisión de Timing en Windows

Windows tiene una resolución de timer de ~15.6ms por defecto, mientras que Linux tiene ~1-5µs.
El daemon incluye optimizaciones específicas para Windows:

- **Delays < 15ms**: Usa spin-wait puro (más CPU, pero preciso)
- **Delays >= 15ms**: Usa sleep + spin-wait híbrido
- **P3min timing**: Tiene tolerancia de 3ms para compensar

### Mejorar precisión (opcional)

Para obtener mejor precisión de timing, ejecutar con prioridad alta:

```powershell
# PowerShell como Administrador
Start-Process -FilePath ".\bmw-diag-daemon.exe" -ArgumentList "" -Verb RunAs

# O desde cmd como Admin:
start /high bmw-diag-daemon.exe
```

### Habilitar timer de alta resolución (avanzado)

Windows 10/11 pueden habilitar timers de 0.5ms con:

```powershell
# Requiere privilegios de administrador
# Establecer resolución de timer a 0.5ms
bcdedit /set useplatformtick yes
bcdedit /set disabledynamictick yes
```

**Nota**: Esto aumenta el consumo de energía. Revertir con:
```powershell
bcdedit /deletevalue useplatformtick
bcdedit /deletevalue disabledynamictick
```
