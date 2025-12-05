#!/usr/bin/env bash
set -e

APP_DIR="/var/www/html"
PATCH_DIR="/opt/laravel-patches"

echo "[php] init start"

# Создаём Laravel, если его ещё нет
if [ ! -f "$APP_DIR/artisan" ]; then
    echo "[php] creating laravel skeleton"
    composer create-project --no-interaction --prefer-dist laravel/laravel:^11 "$APP_DIR"

    cp "$APP_DIR/.env.example" "$APP_DIR/.env"
    sed -i 's|APP_NAME=Laravel|APP_NAME=ISSOSDR|g' "$APP_DIR/.env"
    php "$APP_DIR/artisan" key:generate --force
fi

# Применяем патчи, если есть
if [ -d "$PATCH_DIR" ] && [ "$(ls -A $PATCH_DIR)" ]; then
    echo "[php] applying patches from /opt/laravel-patches"
    rsync -av --no-perms --no-owner --no-group "$PATCH_DIR/" "$APP_DIR/"
fi

# Права (alpine использует www-data:www-data)
chown -R www-data:www-data "$APP_DIR"
chmod -R 775 "$APP_DIR/storage" "$APP_DIR/bootstrap/cache"

# Создаём директорию для логов, если её нет
mkdir -p "$APP_DIR/storage/logs"
chown -R www-data:www-data "$APP_DIR/storage/logs"
chmod -R 775 "$APP_DIR/storage/logs"

# Создаём файл логов, если его нет
touch "$APP_DIR/storage/logs/laravel.log"
chown www-data:www-data "$APP_DIR/storage/logs/laravel.log"
chmod 664 "$APP_DIR/storage/logs/laravel.log"

echo "[php] running migrations (if any)"
php "$APP_DIR/artisan" migrate --force || true

echo "[php] starting php-fpm"
exec php-fpm -F