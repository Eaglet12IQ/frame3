FROM php:8.3-fpm-alpine

# Устанавливаем необходимые пакеты и расширения
RUN apk add --no-cache \
        bash \
        curl \
        git \
        unzip \
        rsync \
        libpq \
        oniguruma \
    && apk add --no-cache --virtual .build-deps \
        $PHPIZE_DEPS \
        postgresql-dev \
    && docker-php-ext-install -j$(nproc) \
        pdo \
        pdo_pgsql \
        bcmath \
        opcache \
    && pecl install redis && docker-php-ext-enable redis \
    && apk del .build-deps

# Composer
COPY --from=composer:2 /usr/bin/composer /usr/bin/composer

# Рабочая директория и права
WORKDIR /var/www/html
RUN mkdir -p /opt/laravel-patches \
    && chown -R www-data:www-data /var/www/html /opt/laravel-patches

# Копируем entrypoint и ПРИНУДИТЕЛЬНО убираем Windows-окончания строк
COPY entrypoint.sh /usr/local/bin/php_entrypoint.sh

# Самый надёжный способ — sed + dos2unix не нужен
RUN sed -i 's/\r$//' /usr/local/bin/php_entrypoint.sh \
    && chmod +x /usr/local/bin/php_entrypoint.sh

# Запуск
CMD ["php-fpm"]
ENTRYPOINT ["/usr/local/bin/php_entrypoint.sh"]