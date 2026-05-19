#!/usr/bin/env bash
# Descarga las librerías de frontend a static/vendor/.
# Ejecutar después de clonar el repo o al actualizar versiones.
set -e

VENDOR="static/vendor"
mkdir -p "$VENDOR/fonts"

echo "Descargando Bootstrap 5.3.3..."
curl -sL "https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css" -o "$VENDOR/bootstrap.min.css"

echo "Descargando Bootstrap Icons 1.11.3..."
curl -sL "https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.3/font/bootstrap-icons.min.css" -o "$VENDOR/bootstrap-icons.min.css"
curl -sL "https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.3/font/fonts/bootstrap-icons.woff2" -o "$VENDOR/fonts/bootstrap-icons.woff2"
curl -sL "https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.3/font/fonts/bootstrap-icons.woff"  -o "$VENDOR/fonts/bootstrap-icons.woff"

echo "Descargando Alpine.js 3.14.3..."
curl -sL "https://cdn.jsdelivr.net/npm/alpinejs@3.14.3/dist/cdn.min.js" -o "$VENDOR/alpine.min.js"

echo "Descargando htmx 2.0.4..."
curl -sL "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js" -o "$VENDOR/htmx.min.js"

echo "Descargando Chart.js 4.5.1..."
curl -sL "https://cdn.jsdelivr.net/npm/chart.js@4.5.1/dist/chart.umd.min.js" -o "$VENDOR/chart.umd.min.js"

echo "Descargando FullCalendar 6.1.11..."
curl -sL "https://cdn.jsdelivr.net/npm/fullcalendar@6.1.11/index.global.min.js" -o "$VENDOR/fullcalendar.global.min.js"

echo "Descargando qrcode.js..."
curl -sL "https://cdn.jsdelivr.net/npm/qrcodejs@1.0.0/qrcode.min.js" -o "$VENDOR/qrcode.min.js"

echo "Descargando html2pdf.js 0.10.1..."
curl -sL "https://cdnjs.cloudflare.com/ajax/libs/html2pdf.js/0.10.1/html2pdf.bundle.min.js" -o "$VENDOR/html2pdf.bundle.min.js"

echo "✓ Listo. Archivos en $VENDOR/"
