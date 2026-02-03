# BMW Diagnostic Tool - Scripts de Instalación

Scripts para instalar y configurar BMW Diagnostic Tool en Debian/Ubuntu.

## Requisitos

- Debian 12 (Bookworm) o Ubuntu 22.04+
- Arquitectura x86_64 (64-bit)
- Conexión a internet
- 5GB de espacio libre en disco
- Adaptador K+DCAN USB (FTDI FT232RL recomendado)

## Instalación Rápida

```bash
# Descargar script
wget https://raw.githubusercontent.com/hessuss14710/bmw-diagnostic-tool/main/scripts/install-bmw-diag.sh

# Hacer ejecutable
chmod +x install-bmw-diag.sh

# Ejecutar como root
sudo ./install-bmw-diag.sh
```

## Opciones del Instalador

```bash
# Instalación normal (reinicia automáticamente)
sudo ./install-bmw-diag.sh

# Sin reinicio automático
sudo ./install-bmw-diag.sh --no-reboot

# Con herramientas de desarrollo (Rust, Node.js)
sudo ./install-bmw-diag.sh --dev

# Ver ayuda
./install-bmw-diag.sh --help
```

## Qué instala el script

1. **Dependencias del sistema**
   - libgtk-3-0, libwebkit2gtk-4.1-0
   - Herramientas USB y serie (usbutils, minicom, screen)

2. **Reglas UDEV**
   - Permisos automáticos para adaptadores K+DCAN
   - Symlink `/dev/kdcan`
   - Deshabilita ModemManager para evitar conflictos

3. **BMW Diagnostic Tool**
   - Descarga e instala el paquete .deb
   - Crea acceso directo en escritorio

4. **Utilidades**
   - `kdcan-test`: Diagnóstico del adaptador
   - `kdcan-monitor`: Monitor serie rápido

## Desinstalación

```bash
sudo ./uninstall-bmw-diag.sh
```

## Solución de Problemas

### El adaptador no se detecta

```bash
# Verificar USB
lsusb | grep -i "ft232\|ch340"

# Recargar módulos
sudo modprobe ftdi_sio
sudo modprobe ch341

# Ver errores del kernel
dmesg | tail -20
```

### Permiso denegado

```bash
# Verificar grupos
groups

# Añadir usuario a dialout (requiere reinicio de sesión)
sudo usermod -aG dialout $USER
```

### Verificar instalación

```bash
# Test completo del adaptador
kdcan-test

# Ver logs de instalación
cat /var/log/bmw-diag-install.log
```

## Adaptadores Soportados

| Chip | Vendor ID | Product ID |
|------|-----------|------------|
| FTDI FT232RL | 0403 | 6001 |
| FTDI FT232R | 0403 | 6015 |
| CH340/CH341 | 1a86 | 7523 |
| CP2102 | 10c4 | ea60 |

## Archivos Importantes

```
/usr/bin/bmw-diag                    # Aplicación
/etc/udev/rules.d/99-kdcan.rules     # Reglas del adaptador
/usr/local/bin/kdcan-test            # Script de diagnóstico
/usr/local/bin/kdcan-monitor         # Monitor serie
~/.local/share/com.bmw-diag.app/     # Datos de usuario
  └── bmw_diag.db                    # Base de datos SQLite
/var/log/bmw-diag-install.log        # Log de instalación
```

## Licencia

MIT License - Ver LICENSE en el repositorio principal.
